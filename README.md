<br>
<p align="center">
    <img src="assets/kumo.svg" height="100">
    <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/voidentente/kumo/main/assets/kumo-light.svg">
        <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/voidentente/kumo/main/assets/kumo-dark.svg">
        <img src="https://raw.githubusercontent.com/voidentente/kumo/main/assets/kumo-dark.svg" height="100">
    </picture>
</p>

## Todo

Kumo development is currently waiting for Bevy 0.10.1 to fix some critical bugs.

## Supported Platforms

Kumo should run on any AMD64/AARCH64 platform that supports Vulkan, including Windows, Linux, and MacOS.

## API Integration

### DeviantArt

To use Kumo with DeviantArt's official API, you must register Kumo as an application in your profile.

Goto https://www.deviantart.com/developers/ where you can register an application.
You can name it whatever you want.

In the application settings, you must add `http://localhost:11211` to "OAuth2 Redirect URI Whitelist",
and keep the grant type at `Authorization Code`. You can find your account's `client_id` and `client_secret`
then at https://www.deviantart.com/developers/.

In Kumo's settings, you can then add your `client_id` and `client_secret`.
To then authenticate with DeviantArt, a browser is required.
Kumo will open DeviantArt's OAuth2 page and perform the authentication for you.
