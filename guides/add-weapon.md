# Adding a New Weapon to a Class

In this guide, you'll learn the basics of DRG Editor by giving the Scout the
Heavy Autocannon as a primary weapon choice.

First, make sure you have the unpacked game files. If you don't, join the
[DRG Modding discord](https://discord.gg/p4UGSnU) and visit the `how-to-make-mods`
channel. You'll also need a copy of the program. Go to the [releases](/releases)
page and grab the latest version.

Begin by opening the program. You'll be greated with a mostly empty screen,
suggesting you open a file.

![Screenshot of program with no files opened](/images/add-weapon/1.png)

Let's do that. Click "Open" in the top toolbar, then navigate to the character
inventory folder. This will be in `<unpacked files>/FSD/Content/Character/InventoryLists`.
Then, open `BP_ScountInventory.uasset`.

![Screenshot of an open file dialog, showing BP_ScoutInventory.uasset being selected](/images/add-weapon/2.png)

Now that a file is opened, the rest of the program appears. To add the Heavy
Autocannon, you'll first need to import it. In the Imports pane in the
top-left, click "Add Import" and a dialog box will open. The first object you
need to import is the weapon's ID asset. Fill in the fields with the values:

- Class Package: `/Script/CoreUObject`
- Class: `Package`
- Name: `/Game/WeaponsNTools/Autocannon/ID_Autocannon`

Then click "Add" to confirm the new import.

![Screenshot of the "Add Import" dialog, with the fields filled in](/images/add-weapon/3.png)

You also need to import the ID_Autocannon object from the ID asset you just imported.
The values you need for this are:

- Class Package: `/Script/FSD`
- Class: `ItemID`
- Name: `ID_Autocannon`

Additionally, you'll need to set the Outer type to "Import", then select the ID asset
from the previous step as the Outer.

![Screenshot of the "Add Import" dialog, with the Outer dropdown opened](/images/add-weapon/4.png)

With the imports out of the way, you're ready to edit the exported values of the
asset. In the top-right, select the only export "BP_ScountInventory". You'll
see a list of properties appear in the bottom-left. Find the "PrimaryWeapons"
property and select it.

![Screenshot showing the export and PrimaryWeapons property selected](/images/add-weapon/5.png)

Now, click the "Add Element" button in the property editor at the bottom-right.
On the new element (Element 3), change the ObjectProperty type to "Import", then
select "ID_Autocannon" from the dropdown menu.

![Screenshot showing the new element with its value set](/images/add-weapon/6.png)

Make sure to click the save button near the top of the property editor. Next,
click "Save As" in the top toolbar, and navigate to the input directory your
DRGPacker. Then, create the folder hierarchy `Content > Character > InventoryLists`
and save to `BP_ScoutInventory.uasset`.

![Screenshot showing saving the asset](/images/add-weapon/7.png)

Finally, run DRGPacker and copy `new_P.pak` to your game's Pak folder. If you
start Deep Rock Galactic and look at the loadout terminal, you'll see you
can choose the Heavy Autocannon as Scout!

![Screenshot showing Scout holding the Heavy Autocannon](/images/add-weapon/8.png)
