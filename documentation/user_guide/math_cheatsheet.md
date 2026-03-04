# Math Cheatsheet (Markdown + KaTeX)

This quick reference helps you write math in Marco using Markdown + KaTeX syntax.

Inspired by: https://www.upyesp.org/posts/makrdown-vscode-math-notation/

---

## 1) How to write math in Markdown

### Inline math
Use single dollar signs inside a sentence:

- Source: ``The area is $A=\pi r^2$.``
- Output: The area is $A=\pi r^2$.

### Block math
Use double dollar signs on separate lines:

- Source:

```markdown
$$
\sum_{k=1}^{n} k = \frac{n(n+1)}{2}
$$
```

- Output:

$$
\sum_{k=1}^{n} k = \frac{n(n+1)}{2}
$$

---

## 2) Arithmetic

| Meaning | Syntax |
|---|---|
| Add / subtract | `$a+b$, $a-b$` |
| Multiply | `$a\cdot b$, $a\times b$` |
| Divide / fraction | `$a/b$, $\frac{a}{b}$` |
| Power | `$x^2$, $x^n$` |
| Root | `$\sqrt{x}$, $\sqrt[3]{x}$` |
| Plus/minus | `$\pm x$` |
| Modulo | `$a \bmod b$` |

---

## 3) Equality and comparison

| Meaning | Syntax |
|---|---|
| Equals / not equals | `$a=b$, $a\neq b$` |
| Approximate | `$\sin(0.01) \approx 0.01$` |
| Equivalent / proportional | `$a\equiv b$, $a\propto b$` |
| Less/greater | `$a<b$, $a>b$` |
| Less/greater or equal | `$a\le b$, $a\ge b$` |

---

## 4) Algebra essentials

| Meaning | Syntax |
|---|---|
| Absolute value | `$|x|$` |
| Function | `$f(x)=x^2+1$` |
| Delta/change | `$\Delta x$` |
| Summation | `$\sum_{i=1}^{n} i^2$` |
| Product | `$\prod_{i=1}^{n} i$` |
| Binomial | `$\binom{n}{k}$` |

---

## 5) Calculus essentials

| Meaning | Syntax |
|---|---|
| Derivative | `$\frac{d}{dx}f(x)$` |
| Partial derivative | `$\frac{\partial f}{\partial x}$` |
| Definite integral | `$\int_a^b f(x)\,dx$` |
| Limit | `$\lim_{x\to 0} \frac{\sin x}{x}$` |
| Gradient / nabla | `$\nabla f$` |

---

## 6) Probability & statistics

| Meaning | Syntax |
|---|---|
| Probability | `$P(A)$`, `$\Pr(A)$` |
| Conditional probability | `$P(A\mid B)$` |
| Intersection / union | `$A\cap B$, $A\cup B$` |
| Mean / variance | `$\mu$, $\sigma^2$` |
| Standard deviation | `$\sigma$` |

---

## 7) Linear algebra

### Vectors

- `$\vec{v}$`, `$\mathbf{v}$`
- Dot product: `$\mathbf{u}\cdot\mathbf{v}$`
- Cross product: `$\mathbf{u}\times\mathbf{v}$`

### Matrices

```markdown
$$
A=\begin{bmatrix}
1 & 2 \\
3 & 4
\end{bmatrix}
$$
```

Common matrix forms:

- `\begin{pmatrix} ... \end{pmatrix}`
- `\begin{bmatrix} ... \end{bmatrix}`
- `\begin{vmatrix} ... \end{vmatrix}`

---

## 8) Greek letters

| Lowercase | Uppercase |
|---|---|
| `$\alpha$, $\beta$, $\gamma$, $\delta$` | `$\Gamma$, $\Delta$` |
| `$\theta$, $\lambda$, $\mu$, $\pi$, $\sigma$, $\omega$` | `$\Theta$, $\Lambda$, $\Pi$, $\Sigma$, $\Omega$` |

---

## 9) Common environments

### Cases

```markdown
$$
f(x)=\begin{cases}
x^2 & \text{if } x\ge 0 \\
-x & \text{if } x<0
\end{cases}
$$
```

### Aligned equations

```markdown
$$
\begin{align}
a+b &= c \\
d+e &= f
\end{align}
$$
```

---

## 10) Marco / KaTeX tips

- Prefer standard delimiters: `$...$` for inline and `$$...$$` for display blocks.
- For multiline environments, both `\\` and `\cr` are accepted by KaTeX.
- If line breaks behave unexpectedly in a specific renderer, try `\cr` as an alternative separator.
- Use `\text{...}` for regular text inside math.
- Keep expressions compact in inline math; use block math for long formulas.

---

## 11) Copy-ready snippets

### Quadratic formula

```markdown
$$
x=\frac{-b\pm\sqrt{b^2-4ac}}{2a}
$$
```

### Euler identity

```markdown
$$
e^{i\pi}+1=0
$$
```

### Gaussian integral

```markdown
$$
\int_{-\infty}^{\infty} e^{-x^2}\,dx=\sqrt{\pi}
$$
```
