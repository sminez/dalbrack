# TODO

- assign weights to actions and may a weighted choice rather than just first returned
- FollowPath needs to be able to stop if the actor has encountered new information that
  should cause it to update its behaviour
  - A simple start would be to track the entities in FOV and then stop if something new
    enters range?
- more map generation algorithms
  - design/notes.md has links to some references
- UI
- player / creature stats

### Light Map
- maybe try allowing more permissive illumination if the light source is between the player
  and the object it is hitting?
  - This would need to be done while building the map at the source level
- update colors for actors and items based on the light map as well as map tiles

### Blitting
The current behaviour just looks for:
  - the map
  - entities containing a tile & position
  - entities containing text & position

This works to get started but doesn't enforce a stacking order and needs to be smarter.
At the very least it should be something like:
  - the map
  - environmental elements
  - items
  - creatures
  - particles & effects
  - free text
  - the UI
