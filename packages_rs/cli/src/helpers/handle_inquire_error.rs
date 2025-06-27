use inquire::InquireError;

pub fn handle_inquire_error(err: InquireError) -> ! {
  match err {
    InquireError::OperationCanceled | InquireError::OperationInterrupted => {
      std::process::exit(1);
    }
    InquireError::NotTTY => {
      println!(
        "The terminal does not support interactive prompts.\
        Please provide the necessary values using command line arguments."
      );
      
      std::process::exit(1);
    }
    _ => {
      println!("Could not process your input.");
      std::process::exit(1);
    }
  }
}
