pub mod lexer;
pub mod parser;
pub mod utils;

fn main() {
    let path = "atlas_core\\src\\test.atlas";
    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => panic!("Problem opening the file: {:?}", error),
    };
    let res = lexer::Lexer::tokenize(path, &contents);
    match res {
        Ok(tokens) => {
            for token in tokens {
                print!("{:?}, ", token.kind());
            }
        }
        Err(error) => panic!("Problem opening the file: {:?}", error),
    }
}

#[macro_export]
macro_rules! map {
    (&key: ty, &val: ty) => {
        {
            let map: HashMap<&key, &val> = HashMap::new();
            map
        }
    };
    ($($key:expr => $val:expr),*) => {
        {
            let mut map = HashMap::new();
            $(map.insert($key, $val);)*
            map
        }
    }
}
