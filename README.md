# Project shows how to handle Facebook and Google token verification in Rust.

## How does it work?
* All magic happens in this [file](https://github.com/jbham/actix-fb-google-login/blob/master/src/auth.rs) on this [line](https://github.com/jbham/actix-fb-google-login/blob/74a99d73199d59f1a1c16efcad57057a56f75f80/src/auth.rs#L73)
* Next, include ```AuthenticatedUser``` struct in your routes like [this on line 9](https://github.com/jbham/actix-fb-google-login/blob/master/src/user/routes.rs)
* Under the hood, it uses ```google-jwt-verify``` library to verify Google Sign-in token and uses ```reqwest``` to verify Facebook token. Once verified, it stores the token details in Redis with auto expiration time based on the token expiration time.

## Note: Instead of using google-jwt-verify crate, this project uses a local forked copy of it. I simply modified to handle Async mutation in the library. Similar changes were submitted by other user. All credits goes to the author of this library. 

