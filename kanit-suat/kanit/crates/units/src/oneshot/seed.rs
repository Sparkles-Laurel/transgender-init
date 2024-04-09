use std::cmp::Ordering;
use std::mem::size_of;
use std::os::unix::fs::PermissionsExt;

use async_fs::File;
use async_trait::async_trait;
use blocking::unblock;
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use log::info;
use nix::errno::Errno;
use nix::fcntl::{open, OFlag};
use nix::request_code_write;
use nix::sys::stat::Mode;
use nix::sys::time::{TimeSpec, TimeValLike};
use nix::time::{clock_gettime, ClockId};
use nix::unistd::close;

use kanit_common::constants;
use kanit_common::error::{Context, ErrorKind, Result};
use kanit_unit::{Dependencies, Unit};

use crate::oneshot::{Clock, LocalMount};
use crate::unit_name;

pub struct Seed;

const RND_IOC_MAGIC: u8 = b'R';
const RND_IOC_TYPE_MODE: u8 = 0x03;
const MAX_SEED_LEN: usize = 512;

#[repr(C)]
struct RandPoolInfo {
    entropy_count: libc::c_int,
    buf_size: libc::c_int,
    buf: [u8; MAX_SEED_LEN], // defined as u32 but it doesn't *really* matter
}

impl Seed {
    fn untrusted_seed() -> u64 {
        let seconds = clock_gettime(ClockId::CLOCK_REALTIME) // TODO; unblock?
            .unwrap_or_else(|_| {
                clock_gettime(ClockId::CLOCK_BOOTTIME).unwrap_or(TimeSpec::new(0, 0))
            })
            .num_seconds();

        u64::from_be_bytes(seconds.to_be_bytes())
    }

    async fn trusted_seed() -> Option<u64> {
        if let Ok(mut data) = async_fs::read(constants::KAN_SEED).await {
            data.resize(8, 0);

            // since the vector was resized, this shouldn't fail
            Some(u64::from_be_bytes(data.as_slice().try_into().unwrap()))
        } else {
            None
        }
    }

    async fn write_bytes(bytes: &[u8], trusted: bool) -> Result<()> {
        let buf: [u8; MAX_SEED_LEN] = match bytes.len().cmp(&MAX_SEED_LEN) {
            Ordering::Greater => (&bytes[0..MAX_SEED_LEN]).try_into().unwrap(),
            Ordering::Less => {
                let mut buf = [0u8; MAX_SEED_LEN];

                let view = &mut buf[0..bytes.len()];

                view.copy_from_slice(bytes);

                buf
            }
            Ordering::Equal => bytes.try_into().unwrap(),
        };

        let entropy_count = if trusted { bytes.len() * 8 } else { 0 };

        let info = RandPoolInfo {
            entropy_count: entropy_count as libc::c_int,
            buf_size: bytes.len() as libc::c_int,
            buf, // unwrap: we ensure the size of `bytes`
        };

        unblock(move || {
            let rand_fd = open("/dev/urandom", OFlag::O_RDONLY, Mode::empty())
                .context_kind("failed to open urandom", ErrorKind::Recoverable)?;

            unsafe {
                Errno::result(libc::ioctl(
                    rand_fd,
                    request_code_write!(
                        RND_IOC_MAGIC,
                        RND_IOC_TYPE_MODE,
                        size_of::<libc::c_int>() * 2
                    ),
                    &info as *const RandPoolInfo,
                ))
            }
            .context_kind("failed to add entropy", ErrorKind::Recoverable)?;

            close(rand_fd).context_kind("failed to close urandom", ErrorKind::Recoverable)?;

            Ok(())
        })
        .await
    }
}

#[async_trait]
impl Unit for Seed {
    unit_name!("seed");

    fn dependencies(&self) -> Dependencies {
        Dependencies::new()
            .need(LocalMount.name())
            .after(Clock.name())
            .clone()
    }

    async fn start(&mut self) -> Result<()> {
        info!("seeding random number generator");

        let t_seed = Self::trusted_seed().await;

        let seed = t_seed.unwrap_or_else(Self::untrusted_seed);

        let mut gen = fastrand::Rng::with_seed(seed);

        let bytes = async_fs::read_to_string("/proc/sys/kernel/random/poolsize")
            .await
            .unwrap_or_else(|_| "2048".to_string())
            .parse::<usize>()
            .unwrap_or(2048)
            / 8;

        // preload some bytes
        let mut buf = Vec::with_capacity(bytes);

        gen.fill(buf.as_mut_slice());

        Self::write_bytes(buf.as_slice(), t_seed.is_some()).await?;

        // load previous seed
        if let Ok(mut previous_seed) = async_fs::read(constants::KAN_SEED).await {
            let len = previous_seed.len();

            previous_seed.resize(bytes, 0);

            if bytes > len {
                gen.fill(&mut previous_seed[len..]);
            }

            Self::write_bytes(previous_seed.as_slice(), true).await?;
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("saving random seed");

        let mut f = File::open("/dev/urandom")
            .await
            .context_kind("failed to open urandom", ErrorKind::Recoverable)?;

        let mut buf = [0; 16];

        f.read_exact(&mut buf)
            .await
            .context_kind("failed to save random seed", ErrorKind::Recoverable)?;

        let mut file = File::create(constants::KAN_SEED)
            .await
            .context_kind("failed to open seed file", ErrorKind::Recoverable)?;

        let metadata = file
            .metadata()
            .await
            .context_kind("failed to get metadata", ErrorKind::Recoverable)?;

        metadata
            .permissions()
            .set_mode((Mode::S_IWUSR | Mode::S_IRUSR).bits());

        file.write(&buf)
            .await
            .context_kind("failed to write seed", ErrorKind::Recoverable)?;

        Ok(())
    }
}
