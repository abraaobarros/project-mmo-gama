# projeto-mmo-gama


Energy
shoot takes energy
energy recovers through time

spells have diferents ranges and spread effects

spells bounce on walls of the shooting player

You can find shields to protect from others players

- right click to defence
- left click to shoot the spell

Every player has a life
the start is near some other players and you have 3 seconds without
infinity world with multiplayer.




first version:

1 circle, multiplayer and line shoot
front: phaser
backend: rust


front:
  - render(position) -> players, lookangles, projectiles

backend:
  - connect(player);
  - move(keys);
  - shoot(theta);



## Running the Server

```sh
cd server/
cargo run 0.0.0.0:90001
```
