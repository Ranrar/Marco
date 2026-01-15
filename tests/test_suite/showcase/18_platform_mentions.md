# Platform mentions showcase

Implemented syntax:
- `@username[platform]`
- `@username[platform](Display Name)`

Rendering behavior:
- Supported platforms render as external profile links.
- Unknown platforms render as a non-link styled span.
- Display name is optional; if present, it replaces the shown username.

## Canonical examples

@ranrar[github]

@ranrar[github](Kim)

@john[twitter](John Doe)

@teamsnapchat[snapchat](Team Snapchat)

## Mid-text mentions

Hello @ranrar[github], welcome!

Multiple: @alice[gitlab] and @bob[bitbucket](Bobby)

## Unknown platform (should not link)

@someone[wechat](No Link)

## Safety / parsing boundaries

### Must have `[platform]` to be a mention

This is an email-like thing and should stay literal:

a@b.com

### Inside code spans (must stay literal)

`@ranrar[github](Kim)`

```
@ranrar[github](Kim)
```

### Punctuation adjacency

(@ranrar[github])

@ranrar[github],

@ranrar[github].

## More platforms (incl. Europe-popular)

@ranrar[codeberg]

@Jane_Doe[xing](Jane Doe)

@durov[vk]

@privacy[telegram]

@ranrar[mastodon]

@Pinterest[pinterest]

@rapidseedbox[medium]

@staff[tumblr]

@Quora[quora]

@bsky.app[bluesky](Bluesky)

@9gag[9gag]

@dribbble[dribbble]

@tom[myspace](Myspace Tom)

@zhihu[zhihu]

@likee[likee]

@2[bilibili](Bilibili UID)
