// Documentation JavaScript for Lisp Compiler
document.addEventListener('DOMContentLoaded', function() {
    // Mobile sidebar toggle
    const sidebarToggle = document.querySelector('.sidebar-toggle');
    const body = document.body;
    
    if (sidebarToggle) {
        sidebarToggle.addEventListener('click', function() {
            body.classList.toggle('sidebar-open');
        });
    }
    
    // Smooth scrolling for internal links
    const internalLinks = document.querySelectorAll('a[href^="#"]');
    internalLinks.forEach(link => {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            const targetId = this.getAttribute('href');
            const targetElement = document.querySelector(targetId);
            
            if (targetElement) {
                targetElement.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
                
                // Close sidebar on mobile after navigation
                if (window.innerWidth <= 1024) {
                    body.classList.remove('sidebar-open');
                }
            }
        });
    });
    
    // Highlight current section in sidebar
    function updateActiveNavItem() {
        const sections = document.querySelectorAll('.content-section');
        const navLinks = document.querySelectorAll('.sidebar-nav-link, .sidebar-subnav-link');
        
        let currentSection = '';
        const scrollPos = window.scrollY + 100;
        
        sections.forEach(section => {
            const sectionTop = section.offsetTop;
            const sectionHeight = section.offsetHeight;
            
            if (scrollPos >= sectionTop && scrollPos < sectionTop + sectionHeight) {
                currentSection = '#' + section.id;
            }
        });
        
        navLinks.forEach(link => {
            link.classList.remove('active');
            if (link.getAttribute('href') === currentSection) {
                link.classList.add('active');
            }
        });
    }
    
    // Update active nav item on scroll
    window.addEventListener('scroll', updateActiveNavItem);
    updateActiveNavItem(); // Initial call
    
    // Copy code functionality
    const codeBlocks = document.querySelectorAll('.code-block pre');
    codeBlocks.forEach(block => {
        const copyButton = document.createElement('button');
        copyButton.className = 'copy-button';
        copyButton.textContent = 'Copy';
        copyButton.setAttribute('aria-label', 'Copy code');
        
        const container = block.parentNode;
        container.style.position = 'relative';
        container.appendChild(copyButton);
        
        copyButton.addEventListener('click', function() {
            const code = block.textContent;
            navigator.clipboard.writeText(code).then(() => {
                copyButton.textContent = 'Copied!';
                copyButton.style.background = 'var(--success-color)';
                
                setTimeout(() => {
                    copyButton.textContent = 'Copy';
                    copyButton.style.background = '';
                }, 2000);
            });
        });
    });
    
    // Search functionality (basic)
    function createSearchBox() {
        const searchContainer = document.createElement('div');
        searchContainer.className = 'search-container';
        searchContainer.innerHTML = `
            <input type="text" class="search-input" placeholder="Search documentation..." />
            <div class="search-results"></div>
        `;
        
        const sidebarHeader = document.querySelector('.sidebar-header');
        sidebarHeader.appendChild(searchContainer);
        
        const searchInput = searchContainer.querySelector('.search-input');
        const searchResults = searchContainer.querySelector('.search-results');
        
        searchInput.addEventListener('input', function() {
            const query = this.value.toLowerCase().trim();
            
            if (query.length < 2) {
                searchResults.innerHTML = '';
                searchResults.style.display = 'none';
                return;
            }
            
            const sections = document.querySelectorAll('.content-section');
            const results = [];
            
            sections.forEach(section => {
                const title = section.querySelector('h2').textContent;
                const content = section.textContent.toLowerCase();
                
                if (content.includes(query)) {
                    results.push({
                        title: title,
                        id: section.id,
                        snippet: extractSnippet(section.textContent, query)
                    });
                }
            });
            
            displaySearchResults(results, searchResults);
        });
    }
    
    function extractSnippet(text, query) {
        const index = text.toLowerCase().indexOf(query.toLowerCase());
        const start = Math.max(0, index - 50);
        const end = Math.min(text.length, index + 100);
        
        let snippet = text.substring(start, end);
        if (start > 0) snippet = '...' + snippet;
        if (end < text.length) snippet = snippet + '...';
        
        return snippet;
    }
    
    function displaySearchResults(results, container) {
        if (results.length === 0) {
            container.innerHTML = '<div class="search-no-results">No results found</div>';
        } else {
            container.innerHTML = results.map(result => `
                <div class="search-result" onclick="location.href='#${result.id}'">
                    <div class="search-result-title">${result.title}</div>
                    <div class="search-result-snippet">${result.snippet}</div>
                </div>
            `).join('');
        }
        
        container.style.display = 'block';
    }
    
    // Initialize search
    createSearchBox();
    
    // Close search results when clicking outside
    document.addEventListener('click', function(e) {
        const searchContainer = document.querySelector('.search-container');
        if (searchContainer && !searchContainer.contains(e.target)) {
            const searchResults = searchContainer.querySelector('.search-results');
            searchResults.style.display = 'none';
        }
    });
    
    // Keyboard shortcuts
    document.addEventListener('keydown', function(e) {
        // Ctrl/Cmd + K to focus search
        if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
            e.preventDefault();
            const searchInput = document.querySelector('.search-input');
            if (searchInput) {
                searchInput.focus();
            }
        }
        
        // Escape to close search
        if (e.key === 'Escape') {
            const searchInput = document.querySelector('.search-input');
            const searchResults = document.querySelector('.search-results');
            if (searchInput && document.activeElement === searchInput) {
                searchInput.blur();
                searchResults.style.display = 'none';
            }
        }
    });
});

// Add additional CSS for search and copy buttons
const additionalStyles = `
.search-container {
    margin-top: 1rem;
    position: relative;
}

.search-input {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 0.375rem;
    font-size: 0.9rem;
    background-color: var(--background-color);
    color: var(--text-primary);
}

.search-input:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 2px rgba(107, 70, 193, 0.1);
}

.search-results {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background-color: var(--background-color);
    border: 1px solid var(--border-color);
    border-radius: 0.375rem;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    max-height: 300px;
    overflow-y: auto;
    z-index: 1001;
    display: none;
}

.search-result {
    padding: 0.75rem;
    cursor: pointer;
    border-bottom: 1px solid var(--border-color);
}

.search-result:hover {
    background-color: var(--hover-bg);
}

.search-result:last-child {
    border-bottom: none;
}

.search-result-title {
    font-weight: 600;
    color: var(--primary-color);
    margin-bottom: 0.25rem;
}

.search-result-snippet {
    font-size: 0.85rem;
    color: var(--text-secondary);
    line-height: 1.4;
}

.search-no-results {
    padding: 1rem;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.9rem;
}

.copy-button {
    position: absolute;
    top: 0.75rem;
    right: 0.75rem;
    padding: 0.25rem 0.5rem;
    background-color: var(--primary-color);
    color: white;
    border: none;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s ease;
}

.code-block:hover .copy-button {
    opacity: 1;
}

.copy-button:hover {
    background-color: var(--primary-dark);
}

.sidebar-nav-link.active,
.sidebar-subnav-link.active {
    background-color: var(--hover-bg);
    border-left-color: var(--primary-color);
    color: var(--primary-color);
    font-weight: 600;
}
`;

// Inject additional styles
const styleSheet = document.createElement('style');
styleSheet.textContent = additionalStyles;
document.head.appendChild(styleSheet);