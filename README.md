# xlabs-api

[xlabs.plutools.pw/api/](https://xlabs.plutools.pw/api/servers)

## Endpoints
All endpoints are json, unless specified otherwise.

- ```/servers```
  - All servers
- ```/servers/<game>```, game = iw4x, iw6x, s1x
  - Servers of specified game

## TODO
- [x] Support all games
- [ ] Cleanup master mod
- [x] Implement challenge
- [ ] get server info async
- [ ] get additional info from ip:port/info if tcp port is open
- [ ] set values to -1 instead of 0 if not found in codinfo