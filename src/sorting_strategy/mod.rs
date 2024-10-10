

pub trait SortingStrategy {
    fn read();

    fn sort();
}


pub struct Standard {}

impl SortingStrategy for Standard {
    
}
