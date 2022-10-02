pub trait Error {
    fn message(&mut self) -> String;
}

impl From<Box<BasicError>> for Box<dyn Error> {
    fn from(b: Box<BasicError>) -> Self {
        b
    }
}

pub struct BasicError {
    msg: String,
}

impl BasicError {
    pub fn new(msg: String) -> Box<Self> {
        Box::new(Self { msg })
    }
}

impl Error for BasicError {
    fn message(&mut self) -> String {
        self.msg.clone()
    }
}