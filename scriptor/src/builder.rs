pub struct Builder {}

struct Resolver;

impl rquickjs::Resolver for Resolver {
    fn resolve<'js>(
        &mut self,
        ctx: rquickjs::Ctx<'js>,
        base: &str,
        name: &str,
    ) -> rquickjs::Result<String> {
        todo!()
    }
}
