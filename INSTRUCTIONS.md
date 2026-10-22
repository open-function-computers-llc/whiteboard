# Whiteboard application

Build me a simple GUI app that is intended for drawing white board sessions during online video screenshares. I think Tauri might be a good fit for this app, but push back on any assumptions if there is a better solution. I considered python with something like pygame but it's old and heavy. I'd like this to be lean and fast to launch, using as little RAM as possible. Tauri might be the wrong frontend, but we need something that can accept drawing with the mouse cursor.

## Current situation

I regularly get into conference calls with customers and have to explain something technical. They are often over Zoom, Google Meet, Microsoft Teams or some other screensharing program. This leads me to opening up the GIMP photo editor just to create a blank image, and use the pencil tool to draw shapes and arrows while I talk through something. Sometimes I have other images open and it's a distraction. Sometimes I have a weird brush enabled and I have to say "hang on" and then click to switch to the pencil tool, change the pixel size, update color, etc. It's annoying and the wrong tool for the job.

## Needs

I want to make this easier, and I want a program that has very little (or none) visual distraction. I'm not looking for professional drawing software, I'm looking to have a blank screen, use the mouse to draw while I talk, and then close it when I'm done, just like a real whiteboard.

## Initial build

Draw a blank white GUI in a standard window. The titlebar should be hidden if possible to give me a fullscreen blank white canvas. Controls should be very limited, choosing between Red, Blue, Black, Green, Purple, and Orange colors for drawing. Clicking with the mouse should begin drawing anywhere that I move the mouse cursor. Holding down space while moving the mouse should pan the visible area of the drawing. Holding down CTRL and mousewheel up should zoom in, and CTRL + mousewheel down should zoom out. Nothing ever needs to be saved. Pressing the ESC button should close the app without warning.

## Tooling

Use git for version control. Initalize a repo and commit regularly as things progress.

Ask questions if you're not sure about something. Assume you're talking to a developer and be technical when asking for direction.

