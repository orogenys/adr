use super::{PageKind, Render, ThemeToggleComponent, escape_html, render_template};

const LAYOUT_TEMPLATE: &str = include_str!("../templates/layout.html");
const THEME_BOOTSTRAP: &str = include_str!("../templates/theme-bootstrap.js");
const MERMAID_BOOTSTRAP: &str = include_str!("../templates/mermaid-bootstrap.js");
const HIGHLIGHT_BOOTSTRAP: &str = r#"
window.addEventListener('DOMContentLoaded', () => {
  if (!document.querySelector('pre code[class*="language-"]')) {
    return;
  }

  const root = document.documentElement;
  const colorScheme = window.matchMedia('(prefers-color-scheme: dark)');
  const lightTheme = 'https://cdn.jsdelivr.net/npm/highlight.js@11.11.1/styles/github.min.css';
  const darkTheme = 'https://cdn.jsdelivr.net/npm/highlight.js@11.11.1/styles/github-dark.min.css';
  let themeLink = document.getElementById('hljs-theme');

  const activeTheme = () => {
    const explicit = root.getAttribute('data-theme');
    if (explicit === 'light' || explicit === 'dark') {
      return explicit;
    }
    return colorScheme.matches ? 'dark' : 'light';
  };

  const applyTheme = () => {
    if (!themeLink) {
      themeLink = document.createElement('link');
      themeLink.id = 'hljs-theme';
      themeLink.rel = 'stylesheet';
      document.head.appendChild(themeLink);
    }
    themeLink.href = activeTheme() === 'dark' ? darkTheme : lightTheme;
  };

  const initialize = () => {
    if (!window.hljs) {
      return;
    }
    document.querySelectorAll('pre code').forEach((block) => {
      window.hljs.highlightElement(block);
    });
  };

  applyTheme();
  new MutationObserver(applyTheme).observe(root, {
    attributes: true,
    attributeFilter: ['data-theme']
  });

  if (typeof colorScheme.addEventListener === 'function') {
    colorScheme.addEventListener('change', applyTheme);
  } else if (typeof colorScheme.addListener === 'function') {
    colorScheme.addListener(applyTheme);
  }

  if (window.hljs) {
    initialize();
    return;
  }

  const script = document.createElement('script');
  script.src = 'https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.11.1/build/highlight.min.js';
  script.onload = initialize;
  document.body.appendChild(script);
});
"#;
const HEADING_LINK_BOOTSTRAP: &str = r#"
window.addEventListener('DOMContentLoaded', () => {
  const toast = document.createElement('div');
  toast.className = 'copy-toast';
  toast.setAttribute('aria-live', 'polite');
  document.body.appendChild(toast);

  let toastTimer = null;
  const showToast = (message) => {
    toast.textContent = message;
    toast.classList.add('copy-toast-visible');
    if (toastTimer) {
      window.clearTimeout(toastTimer);
    }
    toastTimer = window.setTimeout(() => {
      toast.classList.remove('copy-toast-visible');
    }, 1400);
  };

  const copyText = async (text) => {
    if (navigator.clipboard && typeof navigator.clipboard.writeText === 'function') {
      await navigator.clipboard.writeText(text);
      return true;
    }
    return false;
  };

  document.querySelectorAll('.heading-anchor').forEach((anchor) => {
    anchor.addEventListener('click', async (event) => {
      event.preventDefault();
      const hash = anchor.getAttribute('href');
      if (!hash) {
        return;
      }

      const url = new URL(hash, window.location.href).toString();
      if (history && typeof history.replaceState === 'function') {
        history.replaceState(null, '', hash);
      } else {
        window.location.hash = hash;
      }

      const copied = await copyText(url);
      showToast(copied ? 'Section link copied' : 'Section link ready');
    });
  });
});
"#;

pub struct LayoutComponent<'a> {
    pub page_kind: PageKind,
    pub title: &'a str,
    pub heading: &'a str,
    pub heading_html: Option<&'a str>,
    pub subtitle: Option<&'a str>,
    pub subtitle_html: Option<&'a str>,
    pub body: &'a str,
    pub home_label: &'a str,
    pub footer: &'a str,
}

impl Render for LayoutComponent<'_> {
    fn render(&self) -> String {
        let (stylesheet_href, home_href) = match self.page_kind {
            PageKind::Index => ("styles.css", "index.html"),
            PageKind::Adr => ("../../styles.css", "../../index.html"),
        };

        let heading = self
            .heading_html
            .map(str::to_string)
            .unwrap_or_else(|| escape_html(self.heading));

        let subtitle = if let Some(subtitle_html) = self.subtitle_html {
            subtitle_html.to_string()
        } else {
            self.subtitle
                .map(|value| format!("<p class=\"subtitle\">{}</p>", escape_html(value)))
                .unwrap_or_else(|| {
                    "<p class=\"subtitle subtitle-empty\" aria-hidden=\"true\">&nbsp;</p>"
                        .to_string()
                })
        };

        let (theme_script_href, font_inter_href, font_mono_href) = match self.page_kind {
            PageKind::Index => (
                "theme.js",
                "fonts/Inter-Regular.woff2",
                "fonts/JetBrainsMono-Regular.woff2",
            ),
            PageKind::Adr => (
                "../../theme.js",
                "../../fonts/Inter-Regular.woff2",
                "../../fonts/JetBrainsMono-Regular.woff2",
            ),
        };

        render_template(
            LAYOUT_TEMPLATE,
            &[
                ("{{title}}", escape_html(self.title)),
                ("{{stylesheet_href}}", stylesheet_href.to_string()),
                ("{{theme_bootstrap}}", THEME_BOOTSTRAP.to_string()),
                ("{{font_inter_href}}", font_inter_href.to_string()),
                ("{{font_mono_href}}", font_mono_href.to_string()),
                ("{{theme_toggle}}", ThemeToggleComponent.render()),
                ("{{theme_script_href}}", theme_script_href.to_string()),
                ("{{mermaid_bootstrap}}", MERMAID_BOOTSTRAP.to_string()),
                ("{{highlight_bootstrap}}", HIGHLIGHT_BOOTSTRAP.to_string()),
                (
                    "{{heading_link_bootstrap}}",
                    HEADING_LINK_BOOTSTRAP.to_string(),
                ),
                ("{{home_href}}", home_href.to_string()),
                ("{{site_title}}", escape_html(self.home_label)),
                ("{{heading}}", heading),
                ("{{subtitle}}", subtitle),
                ("{{footer}}", self.footer.to_string()),
                ("{{body}}", self.body.to_string()),
            ],
        )
    }
}
