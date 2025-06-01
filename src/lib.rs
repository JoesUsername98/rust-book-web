use std::{ 
    sync::{
        Arc,
        Mutex,
        mpsc,
    },
    thread
};

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool{
    /// Create a new ThreadPool
    /// 
    /// The size is the number of threads in the pool.
    /// 
    /// # Panics
    pub fn new(size: usize) -> ThreadPool{
        assert!( size>0 );

        let ( sender, reciever ) = mpsc::channel();
        let reciever = Arc::new( Mutex::new( reciever ));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size{
            workers.push( Worker::new(id, Arc::clone(&reciever)) );
        }

        ThreadPool{ workers, sender }
    }

    //TODO Replace new with build to not panic
    //pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError>{}
    pub fn execute<F>(&self, f: F)
    where 
        F: FnOnce() + Send +'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker{
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker{
    fn new( id: usize, reciever: Arc<Mutex<mpsc::Receiver<Job>>>   ) -> Worker{
        // TODO Consider std::thread::Builder::spawn to handle cases where the OS cannot give us a new thread.
        let thread = thread::spawn( move || loop {
            let job= reciever
            .lock()
            .expect("Cannot unlock job. Job in poisened state")
            .recv()
            .expect("Thread holding sender end of channel has shutdown.");

            println!("Worker {id} got a job; executing...");
            
            job();

            println!("Worker {id} finished job");
        });
        Worker { id, thread }
    }
}