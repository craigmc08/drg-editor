# DRG Editor

A (currently WIP) tool for editing Deep Rock Galactic `.uasset` and `.uexp`
files.

## How To Use

To get the asset files you want to edit, you'll need some other tools. The
[DRG Modding discord](https://discord.gg/p4UGSnU) has links to these tools and
detailed guides on how to use them and create mods. You can also download
the tools here, though they may be out of date:
[DRG_Modding_Tools_27-4-21.zip](https://cdn.discordapp.com/attachments/799404622682390599/836647075113599066/DRG_Modding_Tools_27-4-21.zip).

Once you have unpacked the games files, open this editor and load an asset.
Make some changes, then save the asset to the input directory of the packing
tool. Then you can pack the mod and load it into the game.

## Known Issues

A lot. There are a lot of issues.

Also, many property types are not editable by this tool. You can resort to hex
editing if necessary, or use the DRGMetaEditor. Hopefully, this program will
be able to replace DRGMetaEditor and almost all hex editing. If you come across
a property type that you want to be able to edit, submit an issue with an
example file from DRG that has the property (if an issue doesn't already exist).
