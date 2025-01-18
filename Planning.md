# Initial thoughts

Plugin needs to be generic to the kinds. They should be debug, clone, reflect.

Offset for HBox should be UVec2 as well.

There should be a BulletTimeClass trait. Basically just needs to implement to f32.

In general bullet time needs to be more expressive. 

- Should be "set base" and "set temp"
- If there are multiple temps at once, they should all tick down simultaneously. The effect will be whichever one is greatest

Can ignore the onscreen stuff

NOT NOW but later could add a PhysicsInactive component to leave burden to specific game

Make physics comps not bundles

Maybe get fancy and make the (replacing bundles) things take IntoIter something

Change the pos in the collision to RxPos and TxPos (I think)