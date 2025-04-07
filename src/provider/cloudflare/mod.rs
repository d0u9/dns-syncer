mod serializer;

mod cloudflare;
pub use cloudflare::Auth;
pub use cloudflare::Cloudflare;

#[cfg(test)]
mod unit_test;
