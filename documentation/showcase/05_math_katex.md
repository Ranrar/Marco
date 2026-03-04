# Math Expressions (KaTeX)

Marco renders math using KaTeX — a fast, native Rust implementation. Both inline and display math are supported with standard LaTeX syntax.

---

## Inline Math

Wrap expressions in single dollar signs: `$...$`

The quadratic formula is $x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$.

Einstein's famous equation: $E = mc^2$.

Euler's identity: $e^{i\pi} + 1 = 0$.

The Pythagorean theorem: $a^2 + b^2 = c^2$.

Probability: The probability of event A given B is $P(A|B) = \frac{P(B|A)P(A)}{P(B)}$.

---

## Display Math

Wrap expressions in double dollar signs on their own lines: `$$...$$`

The Gaussian integral:

$$
\int_{-\infty}^{\infty} e^{-x^2} \, dx = \sqrt{\pi}
$$

The Fourier transform:

$$
\hat{f}(\xi) = \int_{-\infty}^{\infty} f(x)\, e^{-2\pi i x \xi}\, dx
$$

Maxwell's equations in differential form:

$$
\nabla \cdot \mathbf{E} = \frac{\rho}{\varepsilon_0}
\qquad
\nabla \cdot \mathbf{B} = 0
\qquad
\nabla \times \mathbf{E} = -\frac{\partial \mathbf{B}}{\partial t}
\qquad
\nabla \times \mathbf{B} = \mu_0 \mathbf{J} + \mu_0 \varepsilon_0 \frac{\partial \mathbf{E}}{\partial t}
$$

Binomial theorem:

$$
(x + y)^n = \sum_{k=0}^{n} \binom{n}{k} x^{n-k} y^k
$$

---

## Calculus

Derivative definition:

$$
f'(x) = \lim_{h \to 0} \frac{f(x+h) - f(x)}{h}
$$

Integration by parts:

$$
\int u \, dv = uv - \int v \, du
$$

Taylor series:

$$
f(x) = \sum_{n=0}^{\infty} \frac{f^{(n)}(a)}{n!}(x-a)^n
$$

---

## Linear Algebra

Matrix multiplication in inline context: $C = AB$ where $C_{ij} = \sum_k A_{ik} B_{kj}$.

A 3×3 matrix:

$$
A = \begin{pmatrix}
a_{11} & a_{12} & a_{13} \\
a_{21} & a_{22} & a_{23} \\
a_{31} & a_{32} & a_{33}
\end{pmatrix}
$$

Determinant of a 2×2 matrix:

$$
\det(A) = \begin{vmatrix} a & b \\ c & d \end{vmatrix} = ad - bc
$$

---

## Statistics and Probability

Normal distribution PDF:

$$
f(x) = \frac{1}{\sigma\sqrt{2\pi}} \exp\!\left(-\frac{(x-\mu)^2}{2\sigma^2}\right)
$$

Bayes' theorem:

$$
P(A \mid B) = \frac{P(B \mid A)\, P(A)}{P(B)}
$$

---

## Greek Letters and Symbols

Common Greek letters inline: $\alpha$, $\beta$, $\gamma$, $\delta$, $\epsilon$, $\zeta$, $\eta$, $\theta$, $\lambda$, $\mu$, $\nu$, $\xi$, $\pi$, $\rho$, $\sigma$, $\tau$, $\phi$, $\chi$, $\psi$, $\omega$.

Uppercase: $\Gamma$, $\Delta$, $\Theta$, $\Lambda$, $\Xi$, $\Pi$, $\Sigma$, $\Phi$, $\Psi$, $\Omega$.

Operators: $\sum$, $\prod$, $\int$, $\oint$, $\partial$, $\nabla$, $\infty$, $\forall$, $\exists$, $\in$, $\notin$, $\subset$, $\cup$, $\cap$.

---

## Escaping Dollar Signs

Use `\$` to display a literal dollar sign without starting math: the price is \$49.99.

Code spans also prevent math parsing: `$x + y$` stays literal.

---

## Math in Tables

| Formula | Name | Field |
|---------|------|-------|
| $E = mc^2$ | Mass-energy equivalence | Physics |
| $F = ma$ | Newton's second law | Mechanics |
| $PV = nRT$ | Ideal gas law | Thermodynamics |
| $c^2 = a^2 + b^2$ | Pythagorean theorem | Geometry |

---

## Math in Lists

Key identities:

- Euler: $e^{i\pi} + 1 = 0$
- Pythagoras: $c = \sqrt{a^2 + b^2}$
- Quadratic: $x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$
- Logarithm: $\log_b(xy) = \log_b x + \log_b y$
