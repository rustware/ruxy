use tokio::net::TcpStream;
use crate::threads::get_thread_count;

mod multi_threaded;
mod single_threaded;

const QUEUE_SIZE: usize = 8;

pub fn create<Listener, RunnerFut, Task>(listener: Listener)
where
  Listener: FnOnce() -> RunnerFut,
  RunnerFut: Future<Output = ()> + Send + 'static,
  Task: Future<Output = ()> + Send + 'static,
{
  let thread_count = get_thread_count();

  let main_rt = tokio::runtime::Builder::new_current_thread().build().unwrap();

  let (tx, rx) = flume::bounded::<TcpStream>(QUEUE_SIZE);
  
  if thread_count > 1 {
    // We keep the main thread only for load-balancing of incoming sockets and nothing
    // else, so the worker threads count we want to spawn here is (thread_count - 1).
    for _ in 0..thread_count - 1 {
      let rx = rx.clone();
      
      std::thread::spawn(move || {
        let worker_rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        
        worker_rt.block_on(async {
          while let Ok(stream) = rx.recv_async().await {
            
          }
        });
      });
    }
  } else {
    // We're running on a single thread, so we don't need to utilize channels
    // to work-steal the incoming sockets. We'll just spawn a local task.
    main_rt.block_on(async {
      listener();
    });
  }
}
