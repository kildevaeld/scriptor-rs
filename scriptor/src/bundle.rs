use rquickjs::embed;

#[embed(path = "scriptor/lib", public)]
pub mod pipe {}

#[embed(path = "scriptor/lib", public)]
pub mod util {}

#[embed(path = "scriptor/lib", public)]
pub mod tasks {}
