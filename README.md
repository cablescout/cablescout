# CableScout

CableScout adds [OpenID&reg; Connect](https://openid.net/connect/) on top of [WireGuard<sup>&reg;</sup>](https://www.wireguard.com/) for user authentication.

## Main Features

* Lets you use an OIDC provider to authenticate users, instead of managing public keys.
* Rotates client keys every time it connects.

## How It Works

An HTTP/S server is deployed on the same machine that runs the WireGuard server. Its job is to configure the WireGuard server by adding clients that were successfully authenticated and removing expired sessions.

On the client side, a CLI utility uses [wg-quick](https://git.zx2c4.com/wireguard-tools/about/src/man/wg-quick.8) to register a public key with the server, then setting up a connection using the configuration provided by the server.

## Legal

OpenID&reg; is a registered trademark of The OpenID Foundation.

WireGuard<sup>&reg;</sup> is a registered trademark of Jason A. Donenfeld. CableScout is not endorsed, sponsored, or associated with WireGuard<sup>®</sup> or with its community.
