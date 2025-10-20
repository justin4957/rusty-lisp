// Interactive AST Explorer JavaScript

document.addEventListener('DOMContentLoaded', function() {
  // Add click handlers for collapsible nodes
  const headers = document.querySelectorAll('.node-header');

  headers.forEach(header => {
    header.addEventListener('click', function(e) {
      e.stopPropagation();

      const node = this.parentElement;
      const children = node.querySelector('.node-children');
      const toggle = this.querySelector('.toggle');

      if (children) {
        children.classList.toggle('hidden');
        if (toggle) {
          toggle.classList.toggle('collapsed');
        }
      }
    });
  });

  // Add expand/collapse all functionality
  addExpandCollapseButtons();

  // Add search functionality
  addSearchBox();

  // Add node highlighting on hover
  addHoverHighlighting();
});

function addExpandCollapseButtons() {
  const container = document.querySelector('.container');
  const buttonContainer = document.createElement('div');
  buttonContainer.style.cssText = 'margin-bottom: 20px; text-align: center;';

  const expandBtn = document.createElement('button');
  expandBtn.textContent = 'Expand All';
  expandBtn.style.cssText = 'padding: 10px 20px; margin: 0 5px; cursor: pointer; background: #4caf50; color: white; border: none; border-radius: 4px; font-size: 14px;';
  expandBtn.addEventListener('click', () => toggleAll(false));

  const collapseBtn = document.createElement('button');
  collapseBtn.textContent = 'Collapse All';
  collapseBtn.style.cssText = 'padding: 10px 20px; margin: 0 5px; cursor: pointer; background: #f44336; color: white; border: none; border-radius: 4px; font-size: 14px;';
  collapseBtn.addEventListener('click', () => toggleAll(true));

  buttonContainer.appendChild(expandBtn);
  buttonContainer.appendChild(collapseBtn);
  container.insertBefore(buttonContainer, container.querySelector('h1').nextSibling);
}

function toggleAll(collapse) {
  const children = document.querySelectorAll('.node-children');
  const toggles = document.querySelectorAll('.toggle');

  children.forEach(child => {
    if (collapse) {
      child.classList.add('hidden');
    } else {
      child.classList.remove('hidden');
    }
  });

  toggles.forEach(toggle => {
    if (collapse) {
      toggle.classList.add('collapsed');
    } else {
      toggle.classList.remove('collapsed');
    }
  });
}

function addSearchBox() {
  const container = document.querySelector('.container');
  const searchContainer = document.createElement('div');
  searchContainer.style.cssText = 'margin-bottom: 20px;';

  const searchInput = document.createElement('input');
  searchInput.type = 'text';
  searchInput.placeholder = 'Search AST nodes...';
  searchInput.style.cssText = 'width: 100%; padding: 12px; font-size: 16px; border: 2px solid #ddd; border-radius: 4px;';

  searchInput.addEventListener('input', function() {
    const searchTerm = this.value.toLowerCase();
    const nodes = document.querySelectorAll('.ast-node');

    nodes.forEach(node => {
      const text = node.textContent.toLowerCase();
      if (searchTerm === '' || text.includes(searchTerm)) {
        node.style.display = '';
      } else {
        node.style.display = 'none';
      }
    });
  });

  searchContainer.appendChild(searchInput);
  container.insertBefore(searchContainer, container.querySelector('.ast-tree'));
}

function addHoverHighlighting() {
  const nodes = document.querySelectorAll('.ast-node');

  nodes.forEach(node => {
    node.addEventListener('mouseenter', function() {
      this.style.backgroundColor = 'rgba(0, 0, 0, 0.05)';
    });

    node.addEventListener('mouseleave', function() {
      this.style.backgroundColor = '';
    });
  });
}
