//  Welcome to Ruxy! 🎉
//
//  The `ruxy::app!` macro below powers your entire app and is a good place for config.
//
//  Ready to build? Head to the `routes/` directory and start hacking something great.
//    
//  Want the full picture? Start here: https://ruxy.dev/docs/getting-started
//
//  Safe. Fast. Fearless. Have fun. 🤘

ruxy::app! {

    async: {
        // Set this to false to disable async. OS threads will be used instead.
        // More information: https://ruxy.dev/docs/without-async
        enabled: true,
    },
  
    partytown: {
        // Experimental support for Partytown.
        // More information: https://ruxy.dev/docs/partytown
        enabled_experimental: false,
    },
  
    // Faster static matching for known headers, switches to `phf` when more than 30 items
    // More information: https://ruxy.dev/docs/static-matching-and-perfect-hash-function
    known_headers: {
        enabled_experimental: false,
        known_header_names: [
            "X-Test-Header",
        ],
    },
  
    // Faster static matching for known cookies, switches to `phf` when more than 30 items
    // More information: https://ruxy.dev/docs/static-matching-and-perfect-hash-function
    known_cookies: {
        enabled_experimental: false,
        known_cookie_names: [
            "session_id",
        ],
    },

}
