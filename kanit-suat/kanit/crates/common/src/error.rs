use std::fmt::{self, Debug, Display, Formatter};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum ErrorKind {
    Recoverable,
    Unrecoverable,
}

type LazyContext = dyn Send + Fn() -> String;

pub struct Error {
    kind: ErrorKind,
    context: Option<Box<LazyContext>>,
    original: Box<dyn std::error::Error + Send>,
}

impl Error {
    fn new<S: Display + Send + 'static>(
        kind: ErrorKind,
        context: Option<S>,
        original: Box<dyn std::error::Error + Send>,
    ) -> Self {
        Self {
            context: context.map(|n| Box::new(move || n.to_string()) as Box<LazyContext>),
            kind,
            original,
        }
    }

    pub fn is_recoverable(&self) -> bool {
        matches!(self.kind, ErrorKind::Recoverable)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(ref context) = self.context {
            write!(f, "{}: {}", context(), self.original)
        } else {
            write!(f, "{}", self.original)
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("original", &self.original)
            .finish()
    }
}

impl<E: std::error::Error + Send + 'static> From<E> for Error {
    fn from(error: E) -> Self {
        Self::new::<&str>(ErrorKind::Unrecoverable, None, Box::new(error))
    }
}

#[derive(Debug)]
pub struct StaticError(pub &'static str);

impl Display for StaticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StaticError {}

pub struct WithError(Box<LazyContext>);

impl WithError {
    pub fn with<F: Send + Fn() -> String + 'static>(e: F) -> Self {
        Self(Box::new(e) as Box<LazyContext>)
    }
}

impl Display for WithError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0())
    }
}

impl Debug for WithError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error").finish()
    }
}

impl std::error::Error for WithError {}

pub trait Context<T>: Sized {
    fn context_kind<C: Display + Send + 'static>(self, context: C, kind: ErrorKind) -> Result<T>;
    fn with_context_kind<C: Send + Fn() -> String + 'static>(
        self,
        context: C,
        kind: ErrorKind,
    ) -> Result<T> {
        self.context_kind(WithError::with(context), kind)
    }
    fn context<C: Display + Send + 'static>(self, context: C) -> Result<T> {
        self.context_kind(context, ErrorKind::Unrecoverable)
    }
    fn with_context<C: Send + Fn() -> String + 'static>(self, context: C) -> Result<T> {
        self.with_context_kind(context, ErrorKind::Unrecoverable)
    }
    fn kind(self, kind: ErrorKind) -> Result<T>;
}

impl<T> Context<T> for Option<T> {
    fn context_kind<C: Display + Send + 'static>(self, context: C, kind: ErrorKind) -> Result<T> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Error::new(
                kind,
                Some(context),
                Box::new(StaticError("attempted to unwrap 'None' value")),
            )),
        }
    }

    fn kind(self, kind: ErrorKind) -> Result<T> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Error::new::<&str>(
                kind,
                None,
                Box::new(StaticError("attempted to unwrap 'None' value")),
            )),
        }
    }
}

impl<T, E: std::error::Error + Send + 'static> Context<T> for Result<T, E> {
    fn context_kind<C: Display + Send + 'static>(self, context: C, kind: ErrorKind) -> Result<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::new(kind, Some(context), Box::new(e))),
        }
    }

    fn kind(self, kind: ErrorKind) -> Result<T> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(Error::new::<&str>(kind, None, Box::new(e))),
        }
    }
}
