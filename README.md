# Authenticating to Minecraft with the Microsoft Authentication Scheme from Rust
This binary showcases an implementation of the microsoft authentication scheme for Minecraft,
written in Rust.

# Requirements
You need to obtain a client id and client secret by creating an [Azure application]. Steps on obtaining the client id and
client secret will be seen in the next section.

You will also need to provide a redirect uri. This binary assumes that you've set the redirect uri in Azure to be
`localhost`.

A port is also needed. If the port isn't given or is invalid, it will try to parse it from the redirect uri. If it
doesn't exist in the redirect uri, it is set to 80.

In order to input these requirements, you have two methods:
  * Input the variable using environment variables while calling `cargo run`, or
  * Use a `.env` file, placing it within this directory.

Here are the variables needed.

  * `CLIENT_ID`: The client id you got in your azure application.
  * `CLIENT_SECRET`: The client secret you got in your azure application.
  * `REDIRECT_URI`: The redirect uri you gave in your azure application.
  * `PORT` (Optional): The port you gave in your redirect uri. If not given, it is inferred.

# Steps on obtaining a client id and a client secret from Azure
1. Visit portal.azure.com and input your credentials.
2. From the search bar in the top middle of the screen, search for 'Azure Active Directory'.
3. From the sidebar, click on 'App registrations' from the 'Manage' section.
4. Click on 'New Registration' in the menu bar.
5. Set the name of your application to anything you want.
6. Set the supported account type to 'Personal Microsoft accounts only'.
7. Set the type of the Redirect URI to 'Web', and input your redirect uri there.
8. Copy the client id to a safe place.
9. Click on 'Add a certificate or secret' under 'Client credentials'.
10. Click on 'New client secret' under 'Client secrets'.
11. Click on add, optionally adding a description.
12. Copy the client secret to a safe place __immediately__, as the first three digits will only be shown to you afterwards.
13. You should now have the client id and client secret.

# Running it

1. Install Rust from their website: www.rust-lang.org
2. Clone this repository:

```shell
$ git clone https://github.com/ALinuxPerson/mcsoft-auth.git
```

3. Provide the environment variables above in the Requirements section.
4. Run it via `cargo run`.

If all goes well, the first that should pop up in your terminal is this:

```
Now awaiting code.
```

You should have gotten your default browser to open a link to the microsoft oauth page. If nothing popped and an error
occurred in your terminal, you should be able to get the link from there.

Follow the instructions on the link, and then you should get a message on your browser as text like this:

```
Successfully received query
```

Going back to your terminal, you should get the following messages in this order:

```
Now getting the access token.
Now authenticating with Xbox Live.
Now getting an Xbox Live Security Token (XSTS).
Now authenticating with Minecraft.
Checking for game ownership.
```

Up to this point, this should be the messages that you'll if you have _at least_ a microsoft account. However, it 
doesn't assume that you have a valid copy of Minecraft.

These are the following possibilities that could happen:
  * If you own minecraft, it succeeds and continues.
  * If you __don't__ have minecraft, one of these messages will show:
    
    ```
    Error: product_minecraft item doesn't exist. do you really own the game?
    ```
    
    or

    ```
    Error: game_minecraft item doesn't exist. do you really own the game?
    ```
    
    Typically, it should be the first error.

Now, if the game ownership check succeeds, the next lines from your terminal should be as follows:

```
Getting game profile.
Congratulations, you authenticated to minecraft from Rust!
access_token=REDACTED username=REDACTED uuid=REDACTED
```

Of course, replacing `REDACTED` with the real, respective values.

# What this can't do

This binary can't:
  * Support skins.
  * Handle errors. If an error occurs while during one of the steps, you'll instead get an esoteric error message, 
    probably from serde complaining that it can't be parsed correctly.

# Technical Information

The technical information on how this binary works internally can be seen in the (unofficial) 
[Microsoft Authentication Scheme] documentation, and also by looking at the source code of this binary.

[Azure application]: https://docs.microsoft.com/en-us/azure/active-directory/develop/quickstart-register-app
[Microsoft Authentication Scheme]: https://wiki.vg/Microsoft_Authentication_Scheme