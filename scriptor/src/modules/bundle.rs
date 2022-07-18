use rquickjs::embed;

#[embed(path = "lib", path = "scriptor/lib", public)]
pub mod pipe {}

#[embed(path = "lib", path = "scriptor/lib", public)]
pub mod util {}

#[embed(path = "lib", path = "scriptor/lib", public)]
pub mod tasks {}
