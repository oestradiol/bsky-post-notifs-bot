use std::str::FromStr;

mod environment;
pub use environment::*;

/// Utility to attempt leaking a Box to your desired static reference type.
fn try_leak<ToLeak, R: ?Sized>(to_leak: ToLeak) -> Option<&'static R>
where
  Box<R>: TryFrom<ToLeak>,
{
  let leaked: &'static R = Box::<R>::try_from(to_leak).ok().map(Box::leak)?;
  Some(leaked)
}

/// Useful when you want to handle the Option yourself, and do not want the
/// result to be leaked.
///
/// The leaking version of this is `var_opt`.
fn owned_var_opt<T: FromStr>(name: &'static str) -> Option<T> {
  std::env::var(name)
    .ok()
    .filter(|s| s.len() > 0)?
    .parse::<T>()
    .ok()
}

/// Useful when your program requires a variable to be defined and cannot provide a
/// default alternative, but you do not want the parsed result to be leaked/static ref.
/// E.g.: Any Copy type. Not worth leaking.
///
/// The leaking version of this is `var`.
///
/// # Panics
/// When the environment variable is not found or when the parsing fails for T.
fn owned_var<T: FromStr>(name: &'static str) -> T {
  owned_var_opt(name)
    .unwrap_or_else(|| panic!("Couldn't find or parse env variable {name} for given type"))
}

/// Useful when you want to provide a default value for the environment variable,
/// but you do not want the parsed result to be leaked or static.
/// E.g.: Any Copy type. Not worth leaking.
///
/// The leaking version of this function is `var_or`.
fn owned_var_or<T: FromStr>(name: &'static str, default: T) -> T {
  owned_var_opt(name).unwrap_or(default)
}

/// Useful when you want to handle the Option yourself.
///
/// # Leaks
/// This method will leak the parsed value, if any.
fn var_opt<Parsed: FromStr, R: ?Sized>(name: &'static str) -> Option<&'static R>
where
  Box<R>: TryFrom<Parsed>,
{
  try_leak(owned_var_opt::<Parsed>(name)?)
}

/// Useful when your program requires a variable to be defined and cannot
/// provide a default alternative.
///
/// # Leaks
/// This method will leak the parsed value.
///
/// # Panics
/// When the environment variable is not found or when the parsing fails for R.
fn var<Parsed: FromStr, R: ?Sized>(name: &'static str) -> &'static R
where
  Box<R>: TryFrom<Parsed>,
{
  var_opt(name)
    .unwrap_or_else(|| panic!("Couldn't find or parse env variable {name} for given type"))
}

/// Useful when you want to provide a default value for the environment variable,
/// and you have a static reference to your default value.
/// E.g.: A string literal that is stored in the binary.
///
/// # Leaks
/// This method will leak the parsed value.
fn var_or<Parsed: Into<Box<R>> + FromStr, R: ?Sized>(
  name: &'static str,
  default: &'static R,
) -> &'static R
where
  Box<R>: TryFrom<Parsed>,
{
  var_opt(name).unwrap_or(default)
}

/// Useful when you want to provide a default value for the environment variable,
/// but you don't have a static reference to the value.
/// E.g.: An owned `PathBuf` -> A `&'static Path`.
///
/// # Leaks
/// This method will leak the parsed or the default value.
fn var_or_else<Parsed: Into<Box<R>> + FromStr + Sized, R: ?Sized, V: FnOnce() -> Parsed>(
  name: &'static str,
  default: V,
) -> &'static R
where
  Box<R>: TryFrom<Parsed>,
{
  var_or(name, Box::leak(default().into()))
}
