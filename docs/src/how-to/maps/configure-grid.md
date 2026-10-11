# Set the Grid

The grid tells Mimir how large one 5-foot square is on the map image. Tokens snap to it, and sight ranges in feet use it.

## UVTT Files

A UVTT file holds its grid. Mimir uses it. There is nothing to set.

## Image Files

You set the grid when you upload the image.

1. Find the size of one square of the map in pixels. Many map makers give it, for example "70 px per square". Or open the image in an image editor and measure one square.
2. Upload the map. See [Upload a Map](./upload-map.md).
3. In **Grid: pixels a square**, type the size. The value is from 10 to 1000.
4. Click **Upload**.

The grid starts at the top-left corner of the image.

## Check the Grid

1. Open the map on the DM map.
2. Compare the grid lines of Mimir with the squares drawn on the map.
3. Place a token. It must fill one square.

## If the Grid Does Not Fit

You cannot change the grid of a map after the upload.

1. Delete the map.
2. Upload it again with the right **Grid: pixels a square**.

If the squares of the image do not start at the top-left corner, crop the image first, or use the UVTT version of the map.

## Token Sizes

A token covers squares by creature size: Tiny half a square, Small and Medium one square, Large 2×2, Huge 3×3, Gargantuan 4×4.

## See Also

- [Upload a Map](./upload-map.md)
- [Place Tokens](./place-tokens.md)
- [Map Formats](../../explanation/map-formats.md)
