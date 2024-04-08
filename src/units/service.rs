// service.rs - Service target type
// Service targets are the most basic targets.
// They are used for starting and stopping services.
//

pub struct Service {
    name: String,
    description: String,
    exec_start: String,
    exec_stop: String,
    exec_reload: String,
    exec_restart: String,
    dependencies: Vec<String>,
    conflicts: Vec<String>,
    before: String,
    after: String,
    wanted_by: String,
}

// import the libraries that control processes,
// start new ones and stop them.

impl Target for Service {
    fn conduct(&self, args: InitArgs) -> Result<(), String> {
        // start the specified program.
        // first, split the exec_start string into a vector.
        let exec_start: Vec<&str> = self.exec_start.split(" ").collect();
        // now, start the program.
        let mut child = Command::new(exec_start[0])
            .args(&exec_start[1..])
            .spawn()
            .expect("Failed to start the program.");
        // wait for the program to finish.
        child.wait().expect("Failed to wait for the program to finish.");
        // return Ok.
        Ok((), "Service started successfully.".to_string())
    }
}