use crate::*;

pub struct ApiContext<I: Interface> {
    interface: I,
}

impl<I: Interface> ApiContext<I> {
    pub fn new(interface: I) -> Self {
        Self { interface }
    }

    pub async fn init(&self, options: InitOptions) -> miette::Result<()> {
        init::init(&self.interface, options).await
    }
}
