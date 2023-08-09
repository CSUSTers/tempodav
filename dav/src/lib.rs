#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DavConfig {}

impl Default for DavConfig {
    fn default() -> Self {
        DavConfig {}
    }
}

impl DavConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> DavServer {
        DavServer::new(self)
    }
}

#[derive(Debug)]
pub struct DavServer {
    config: DavConfig,
}

impl DavServer {
    pub fn new(config: DavConfig) -> Self {
        DavServer { config }
    }

    pub fn builder() -> DavConfig {
        DavConfig::new()
    }

    pub async fn run(&self) {
        // TODO: implement
        unimplemented!()
    }
}
