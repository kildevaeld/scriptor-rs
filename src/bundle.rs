use rquickjs::embed;

#[embed(path = "lib", public)]
pub mod pipe {}

#[embed(path = "lib", public)]
pub mod util {}

#[embed(path = "lib", public)]
pub mod tasks {}
