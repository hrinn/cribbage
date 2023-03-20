pub enum Frame {
    Name(String), // Client sends name to server
    Start(Vec<String>), // Server tells client game starts, includes list of names

}

