window.addEventListener('DOMContentLoaded', () => {
  if (!document.querySelector('.mermaid')) {
    return;
  }

  const createViewer = () => {
    const modal = document.createElement('div');
    modal.className = 'mermaid-modal';
    modal.hidden = true;
    modal.innerHTML = `
      <div class="mermaid-modal-backdrop" data-mermaid-close="true"></div>
      <div class="mermaid-modal-panel" role="dialog" aria-modal="true" aria-label="Mermaid diagram viewer">
        <div class="mermaid-modal-toolbar">
          <p class="mermaid-modal-title">Diagram viewer</p>
          <button type="button" class="mermaid-modal-close" data-mermaid-close="true">Close</button>
        </div>
        <div class="mermaid-modal-stage"></div>
      </div>
    `;
    document.body.appendChild(modal);

    const stage = modal.querySelector('.mermaid-modal-stage');
    const closeButton = modal.querySelector('.mermaid-modal-close');
    let currentObjectUrl = null;

    const releaseObjectUrl = () => {
      if (!currentObjectUrl) {
        return;
      }
      URL.revokeObjectURL(currentObjectUrl);
      currentObjectUrl = null;
    };

    const close = () => {
      modal.hidden = true;
      document.body.classList.remove('mermaid-modal-open');
      stage.replaceChildren();
      releaseObjectUrl();
    };

    const open = (diagram) => {
      const svg = diagram.querySelector('svg');
      if (!svg) {
        return;
      }

      let markup = new XMLSerializer().serializeToString(svg);
      if (!markup.includes('xmlns="http://www.w3.org/2000/svg"')) {
        markup = markup.replace(
          '<svg',
          '<svg xmlns="http://www.w3.org/2000/svg"'
        );
      }

      releaseObjectUrl();
      currentObjectUrl = URL.createObjectURL(
        new Blob([markup], { type: 'image/svg+xml;charset=utf-8' })
      );

      const image = document.createElement('img');
      image.className = 'mermaid-modal-image';
      image.alt = 'Expanded Mermaid diagram';
      image.src = currentObjectUrl;

      const sourceWidth = svg.getBoundingClientRect().width || 0;
      const preferredWidth = Math.min(
        Math.max(sourceWidth * 1.35, 720),
        window.innerWidth * 0.92
      );
      if (preferredWidth > 0) {
        image.style.width = `${preferredWidth}px`;
      }

      stage.replaceChildren(image);
      modal.hidden = false;
      document.body.classList.add('mermaid-modal-open');
      closeButton.focus();
    };

    modal.addEventListener('click', (event) => {
      if (
        event.target instanceof Element
        && event.target.closest('[data-mermaid-close="true"]')
      ) {
        close();
      }
    });

    document.addEventListener('keydown', (event) => {
      if (!modal.hidden && event.key === 'Escape') {
        close();
      }
    });

    return { open };
  };

  const viewer = createViewer();

  const enableViewer = () => {
    document.querySelectorAll('.mermaid-diagram').forEach((container) => {
      if (container.dataset.mermaidViewer === 'ready') {
        return;
      }

      const diagram = container.querySelector('.mermaid');
      const button = container.querySelector('[data-mermaid-expand]');
      if (!(diagram instanceof HTMLElement) || !(button instanceof HTMLButtonElement)) {
        return;
      }

      if (!diagram.querySelector('svg')) {
        return;
      }

      const open = () => viewer.open(diagram);
      container.dataset.mermaidReady = 'true';
      container.dataset.mermaidViewer = 'ready';
      button.hidden = false;
      button.addEventListener('click', open);
      diagram.setAttribute('role', 'button');
      diagram.setAttribute('tabindex', '0');
      diagram.setAttribute('aria-label', 'Open diagram in a larger view');
      diagram.title = 'Open diagram in a larger view';
      diagram.addEventListener('click', open);
      diagram.addEventListener('keydown', (event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          open();
        }
      });
    });
  };

  const initialize = async () => {
    if (!window.mermaid) {
      return;
    }

    window.mermaid.initialize({
      startOnLoad: false,
      theme: document.documentElement.dataset.theme === 'dark' ? 'dark' : 'default',
      securityLevel: 'loose'
    });

    try {
      await window.mermaid.run({ querySelector: '.mermaid' });
    } catch (error) {
      console.error('Failed to render Mermaid diagrams.', error);
    }

    enableViewer();
    window.requestAnimationFrame(enableViewer);
  };

  if (window.mermaid) {
    initialize();
    return;
  }

  const script = document.createElement('script');
  script.src = 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js';
  script.onload = initialize;
  document.body.appendChild(script);
});
