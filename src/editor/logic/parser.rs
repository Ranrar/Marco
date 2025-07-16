use MarkdownNode::*;

let ast = Document(vec![
    Heading {
        level: 1,
        content: vec![Text("Hello World".into())],
    },
    Paragraph(vec![
        Text("This is ".into()),
        Emphasis(vec![Text("italic".into())]),
        Text(", ".into()),
        Strong(vec![Text("bold".into())]),
        Text(", and ".into()),
        Code("code".into()),
        Text(".".into()),
    ]),
    List {
        ordered: false,
        items: vec![
            ListItem(vec![Paragraph(vec![Text("First item".into())])]),
            ListItem(vec![Paragraph(vec![Text("Second item".into())])]),
        ],
    },
]);
