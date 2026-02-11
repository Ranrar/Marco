# YouTube Video Embed Testing
## Using Rick Astley - Never Gonna Give You Up

All examples use the same video (dQw4w9WgXcQ) to ensure thumbnails load properly.

---

## Test 1: Basic YouTube Embed

[![Never Gonna Give You Up](https://img.youtube.com/vi/dQw4w9WgXcQ/0.jpg)](https://www.youtube.com/watch?v=dQw4w9WgXcQ)

Expected: Clickable YouTube thumbnail image.

---

## Test 2: YouTube Embed with Timestamp

[![Video at 42 seconds](https://img.youtube.com/vi/dQw4w9WgXcQ/0.jpg)](https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42s)

Expected: Clickable thumbnail, link includes timestamp.

---

## Test 3: YouTube Short URL Format

[![Short URL Format](https://img.youtube.com/vi/dQw4w9WgXcQ/0.jpg)](https://youtu.be/dQw4w9WgXcQ)

Expected: Works with youtu.be short links.

---

## Test 4: YouTube Short URL with Timestamp

[![Short URL with time](https://img.youtube.com/vi/dQw4w9WgXcQ/0.jpg)](https://youtu.be/dQw4w9WgXcQ?t=42)

Expected: Short URL with timestamp parameter.

---

## Test 5: Alternative Thumbnail Service (i.ytimg.com)

[![Alt thumbnail service](https://i.ytimg.com/vi/dQw4w9WgXcQ/0.jpg)](https://www.youtube.com/watch?v=dQw4w9WgXcQ)

Expected: Works with i.ytimg.com thumbnail URLs.

---

## Test 6: High Quality Thumbnail (maxresdefault)

[![High quality thumbnail](https://img.youtube.com/vi/dQw4w9WgXcQ/maxresdefault.jpg)](https://www.youtube.com/watch?v=dQw4w9WgXcQ)

Expected: Higher resolution thumbnail (if available).

---

## Test 7: Medium Quality Thumbnail (mqdefault)

[![Medium quality thumbnail](https://img.youtube.com/vi/dQw4w9WgXcQ/mqdefault.jpg)](https://www.youtube.com/watch?v=dQw4w9WgXcQ)

Expected: Medium resolution thumbnail.

---

## Test 8: Non-YouTube Link (Should NOT be Video Embed)

[![Example Image](https://example.com/image.jpg)](https://example.com/page)

Expected: Regular image link, NOT treated as video embed.

---

## Test 9: Multiple Embeds in Same Document

Here's the first video:

[![First embed](https://img.youtube.com/vi/dQw4w9WgXcQ/0.jpg)](https://www.youtube.com/watch?v=dQw4w9WgXcQ)

And here's the same video linked again:

[![Second embed](https://img.youtube.com/vi/dQw4w9WgXcQ/mqdefault.jpg)](https://youtu.be/dQw4w9WgXcQ?t=30)

Expected: Both should render as video embeds.

---

## Test 10: Inline with Text

Check out this video [![inline video](https://img.youtube.com/vi/dQw4w9WgXcQ/0.jpg)](https://www.youtube.com/watch?v=dQw4w9WgXcQ) in the middle of a paragraph.

Expected: Video embed inline with surrounding text.

---

## Notes

- **Default Mode**: Video embeds are DISABLED by default (privacy/safety)
  - Renders as: `<a href="video-url"><img src="thumbnail" /></a>`
  - Clickable thumbnail that opens YouTube in browser
  
- **Enabled Mode**: When `enable_video_embeds: true` in RenderOptions
  - Renders as: `<iframe>` embedded video player
  - Uses youtube-nocookie.com for privacy
  - Video plays directly in preview

Current Marco/Polo behavior: Falls back to clickable thumbnail (embeds disabled).
