# Project shows how to handle Facebook and Google token verification in Rust using Actix-Web.

## How does it work?
* All magic happens in this [file](https://github.com/jbham/actix-fb-google-login/blob/master/src/auth.rs) on this [line](https://github.com/jbham/actix-fb-google-login/blob/74a99d73199d59f1a1c16efcad57057a56f75f80/src/auth.rs#L73)
* Next, include ```AuthenticatedUser``` struct in your routes like [this on line 9](https://github.com/jbham/actix-fb-google-login/blob/master/src/user/routes.rs)
* Under the hood, it uses ```google-jwt-verify``` library to verify Google Sign-in token and uses ```reqwest``` to verify Facebook token. Once verified, it stores the token details in Redis with auto expiration time based on the token expiration time.

## How to use?
* User would authenticate using Google Sign in or Facebook Login in the browser.
  * Browser would submit either google or facebook token in Authorization header of the HTTP request
  * Actix will verify the token and proceed depending on whether token is valid/invalid. If invalid, returns 401 error. If valid, it proceeds with rest of the steps.
  * For this App, all GET requests are non-authenticated so they are allowed without any verification.



#### Note: Instead of using google-jwt-verify crate, this project uses a local forked copy of it. I simply modified to handle Async mutation in the library. Similar changes were submitted by other user. Hence, I kept my changes local. All credits go to the author of this library. 

