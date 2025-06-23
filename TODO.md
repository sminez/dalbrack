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
