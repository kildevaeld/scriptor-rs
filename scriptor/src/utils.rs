use rquickjs::{Loader, Resolver};

#[derive(Clone, Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Resolver for Either<L, R>
where
    L: Resolver,
    R: Resolver,
{
    fn resolve<'js>(
        &mut self,
        ctx: rquickjs::Ctx<'js>,
        base: &str,
        name: &str,
    ) -> rquickjs::Result<String> {
        match self {
            Either::Left(left) => left.resolve(ctx, base, name),
            Either::Right(right) => right.resolve(ctx, base, name),
        }
    }
}

impl<L, R, T> Loader<T> for Either<L, R>
where
    L: Loader<T>,
    R: Loader<T>,
{
    fn load<'js>(
        &mut self,
        ctx: rquickjs::Ctx<'js>,
        name: &str,
    ) -> rquickjs::Result<rquickjs::Module<'js, rquickjs::Loaded<T>>> {
        match self {
            Either::Left(left) => left.load(ctx, name),
            Either::Right(right) => right.load(ctx, name),
        }
    }
}
