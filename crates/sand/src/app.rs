//  Welcome to Ruxy! 🎉
//
//  Ready to build? Head to the `routes/` directory and start hacking something great.
//
//  Want the full picture? Start here: https://ruxy.dev/docs/getting-started
//
//  Build fearlessly. Smoke benchmarks. Have fun. 🤘

ruxy::app! {
  
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
