# Using Jun's Garden

A picture for every step. The pictures come from a test copy of the garden with made-up sample
plants, so your plants and names will look different. Big picture = computer, small picture =
phone. The thing to look at has an orange frame.

## 1. Signing in, and your password

Anyone can look at the garden. To add or change plants, sign in.

1. Open your garden's web address. The home page has a login box: type your name (**Jun**) and your
   password and press **Login**. (The same box is on its own page, `/login.php`.)
   @shot 01-login
2. Signed in, the home page says **Welcome back, Jun!**, and the admin pages open for you.
   @shot 03-home-signed-in
3. **Forgot your password, or the first time?** On the login page choose **Forgot password? / First
   time?**. Type your name; a link to set a new password goes to your email.
   @shot 02-forgot

*On the garden computer you can also set it any time: `bash garden.sh password` (or the menu, U, then 5).*

## 2. The admin page

All the plant tools are on one page, in tabs: **Admin** (the address ends in `/streamlit/`). Sign in
first.

1. The tabs along the top each do one job: add a plant, edit one, photos, beds, the wish list, barcodes.
   @shot 04-admin-tabs

## 3. Adding a plant

*New: the first tab, **🌱 Add plant**, asks for six things and lets the local plant AI fill in the
rest. It has its own picture guide: `walkthrough/streamlit/` (English and Korean). The steps below
are the full form, now under **⚙️ Advanced → 🌱 Add Plant (full form)**.*

1. Open the tab **🌱 Add Plant**. Type the plant's common name and its botanical name.
   @shot 05-add-plant
2. Press **Enter**. The next step appears, with the button **📋 Create Record →**.
   @shot 06-add-plant-next
3. Press **📋 Create Record →**. The page checks that the plant is not in the garden already
   (**Step 2**), then asks how to add it (**Step 3**): **⚡ Quick Create** (the basics now, the rest
   later) or **✨ AI Enrich** (the free AI fills in the details). Follow the steps down the page; the
   last button saves the plant, and it appears in the catalogue.
   @shot 06b-add-plant-saved

## 4. Editing a plant

1. Open the tab **✍️ Manual Editor** and choose the plant from the list at the top.
2. Change what you want (for example the company, the bed, the sun it likes) and press **Save** at
   the bottom.
   @shot 07-edit-plant

## 5. Removing a plant (it died, or you gave it away)

The garden keeps the plant's record and its history; you record that it is gone.

1. In **Full Catalog**, tap the plant to open its window.
   @shot 08-plant-window
2. Press **✏️ Edit**. In the green box below the form, choose the event **🗑️ Removed** (or **💀 Died**),
   how many, and save it.
   @shot 09-remove-plant

## 6. Photos

1. To give a plant a photo, or replace one: the admin tab **📸 Manual Image Replace**. Choose the plant,
   pick the photo from your computer or phone, and save. **🗑️ Remove this image** takes a photo away.
   @shot 10-photos-admin
2. The **Gallery** page shows the garden's photos for everyone.
   @shot 11-gallery

## 7. Beds and bed labels (QR codes)

1. **🗺️ Bed Assignment** (admin tab) puts plants into beds: tick the beds for each plant and save.
   @shot 12-beds
2. **Labels**: open `/labels` (signed in). Every bed and every living plant has a label with a QR code.
   Print the page and cut the labels out. Scanning a label opens that plant or bed on the website.
   @shot 13-labels

*The codes keep working when you rename a bed or change the garden's web address.*

## 8. Comments and the guestbook

1. **Guestbook**: at the bottom of the home page anyone can leave a short message.
   @shot 14-guestbook
2. **Plant comments**: in a plant's window, tap its photo. Under the big photo are the comments and a box
   to write one. You get a little bell on the admin pages when someone comments.
   @shot 15-plant-comments

## 9. Search

1. **Quick Search**: type part of a name; matching plants appear as you type.
   @shot 16-quick-search
2. **Full Catalog**: the search box at the top filters the cards; the letters and the bed buttons
   narrow it down too.
   @shot 17-catalog-search

## 10. The phone app

1. On the computer, open `/app/setup`. It has two codes.
   @shot 18-app-setup
2. On the phone, scan the **first** code: it downloads the app. Install it (the first time, Android
   asks you to allow installs from your browser: say yes).
3. Scan the **second** code: the app learns your garden's address. It says so at the bottom of the
   screen.
4. Open the app. Sign in with your garden name and password, or choose **Continue as guest** to look.
   @app 07-en-1-sign-in
5. The catalogue: search at the top, beds below it, then the plants. Tap a plant to open it.
   @app 07-en-2-catalogue
6. A plant: its photo, the facts in groups, and the notes. **‹ Catalog** goes back.
   @app 07-en-3-plant
7. Along the bottom (signed in): **Identify** (photo of a plant), **Tag** (photo of a plant tag),
   **Barcode** (photo of a barcode) and **Receipt** (photo of a receipt).
8. When there is a new version, the app offers **Update**.

*The app speaks English and Korean. It follows the phone's language, or choose one for this app only:
Settings, Apps, Jun's Garden, Language.*

## 11. Changing the garden's web address

One step does everything: the address the garden uses, the labels, the phone app, and the old address
keeps forwarding to the new one, so printed labels and old phones keep working. First add the new name
to your No-IP account.

1. Signed in, open `/labels`. In **Web address**, type the new web name (and a folder if you want one)
   and press **Change the address**. Within a minute the page says it changed.
   @shot 19-address
2. Or on the garden computer: `bash garden.sh webname new.name.org` (the menu: M, then 16), or the
   Windows setup wizard's last page: **Change the web address**.
