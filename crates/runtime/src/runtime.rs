use std::net::SocketAddr;

use tokio::net::TcpListener;

/// This trait is a "bridge" between the runtime and the `app!` macro.
/// Abstract methods are implemented by the `app!` macro, enabling the
/// expanded macro to call into the runtime and vice versa.
/// 
/// Since this trait is used by the `app!` macro, the user code has to
/// be able to import it, but it should never be called by the user
/// directly, thus `#[doc(hidden)]` is applied.
#[doc(hidden)]
pub trait Runtime: Sized {
  fn start(self) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
      // TODO: Make this configurable:
      let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

      let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
          // TODO: Logging
          eprintln!("[ERR] failed to bind to '{}': {}", addr, e);
          std::process::exit(1);
        },
      };

      loop {
        match listener.accept().await {
          Ok((socket, addr)) => {
            // TODO: Metrics
            
            tokio::spawn(async {
              
            });
          },
          Err(e) => {
            // TODO: Logging, metrics
            eprintln!("[ERR] couldn't get client: {}", e);
            continue;
          },
        }
      }
    });


  }
}
