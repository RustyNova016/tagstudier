# Folder configuration

Tagstudier allows for easy management of the folders inside the library. While TagStudio itself doesn't encourage touching the inner files, it has a bunch of use cases still:
- Lesser strain on the filesystem, making folders open much faster than if you shove everything into a single one
- Easier organistion when not using TagStudio
- Allows for synchronizing files to other devices without bringing the whole library 

This is done by making "Folder rules". A folder rule apply a specific set of rules that tagstudier will apply. Rules are added in a `.TagStudio/TSR_folder_rules.toml` file. 

## Order

Here's the order each rules are processed:
- **Sorting**: Each rule is read and applied top to bottom. Any entry moved by a rule cannot be moved by the next ones. This means that folders containing a subset of another one (Ex: Folder "maxwell" being a subset of folder "cat") needs to be before the more general one

## Folder rule syntax

| Name    | Required | Format | Description                                                                                                                                     |
| ------- | -------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| path    | Yes      | String | The path of the folder the rule target                                                                                                          |
| sorting | No       | String | What entries should get sorted in the folder. This is a search string. Any entry that appear in this search sting will be moved to this folder. |

## Example file

```toml
[[folder]]
path = "memes/cat/maxwell"
sorting = "maxwell"

[[folder]]
path = "memes/animals/"
sorting = "(meme cat) or (meme dog)"