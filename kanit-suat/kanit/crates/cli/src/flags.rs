xflags::xflags! {
    cmd kanit {
        /// Teardown and power-off the system.
        cmd poweroff {
            /// Force a power-off, not performing a teardown.
            optional -f, --force
        }
        /// Teardown and reboot the system.
        cmd reboot {
            /// Force a reboot, not performing a teardown.
            optional -f, --force
        }
        /// Teardown and halt the system.
        cmd halt {
            /// Force a halt, not performing a teardown.
            optional -f, --force
        }
        /// Teardown and reboot the system via kexec.
        cmd kexec {
            /// Force a reboot via kexec, not performing a teardown.
            optional -f, --force
        }
        /// Print unit startup times.
        cmd blame {
            // Print units sorted by startup time.
            optional -s, --sorted
        }
        /// Service related utilities.
       cmd service {
            /// Enable a unit at the specified runlevel.
            cmd enable {
                /// The name of the unit.
                required unit: String
                /// The runlevel to enable the service at.
                optional runlevel: usize
            }
            /// Disable a unit at the specified runlevel.
            cmd disable {
                /// The name of the unit.
                required unit: String
                /// The runlevel to enable the service at.
                optional runlevel: usize
            }
            /// List all enabled units.
            cmd list {
                /// Shows the individual unit groups.
                optional -p, --plan
            }
       }
    }
}
