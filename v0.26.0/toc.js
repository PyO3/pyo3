// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded "><a href="getting-started.html"><strong aria-hidden="true">1.</strong> Getting started</a></li><li class="chapter-item expanded "><a href="rust-from-python.html"><strong aria-hidden="true">2.</strong> Using Rust from Python</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="module.html"><strong aria-hidden="true">2.1.</strong> Python modules</a></li><li class="chapter-item expanded "><a href="function.html"><strong aria-hidden="true">2.2.</strong> Python functions</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="function/signature.html"><strong aria-hidden="true">2.2.1.</strong> Function signatures</a></li><li class="chapter-item expanded "><a href="function/error-handling.html"><strong aria-hidden="true">2.2.2.</strong> Error handling</a></li></ol></li><li class="chapter-item expanded "><a href="class.html"><strong aria-hidden="true">2.3.</strong> Python classes</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="class/protocols.html"><strong aria-hidden="true">2.3.1.</strong> Class customizations</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="class/object.html"><strong aria-hidden="true">2.3.1.1.</strong> Basic object customization</a></li><li class="chapter-item expanded "><a href="class/numeric.html"><strong aria-hidden="true">2.3.1.2.</strong> Emulating numeric types</a></li><li class="chapter-item expanded "><a href="class/call.html"><strong aria-hidden="true">2.3.1.3.</strong> Emulating callable objects</a></li></ol></li><li class="chapter-item expanded "><a href="class/thread-safety.html"><strong aria-hidden="true">2.3.2.</strong> Thread safety</a></li></ol></li></ol></li><li class="chapter-item expanded "><a href="python-from-rust.html"><strong aria-hidden="true">3.</strong> Calling Python from Rust</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="types.html"><strong aria-hidden="true">3.1.</strong> Python object types</a></li><li class="chapter-item expanded "><a href="exception.html"><strong aria-hidden="true">3.2.</strong> Python exceptions</a></li><li class="chapter-item expanded "><a href="python-from-rust/function-calls.html"><strong aria-hidden="true">3.3.</strong> Calling Python functions</a></li><li class="chapter-item expanded "><a href="python-from-rust/calling-existing-code.html"><strong aria-hidden="true">3.4.</strong> Executing existing Python code</a></li></ol></li><li class="chapter-item expanded "><a href="conversions.html"><strong aria-hidden="true">4.</strong> Type conversions</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="conversions/tables.html"><strong aria-hidden="true">4.1.</strong> Mapping of Rust types to Python types</a></li><li class="chapter-item expanded "><a href="conversions/traits.html"><strong aria-hidden="true">4.2.</strong> Conversion traits</a></li></ol></li><li class="chapter-item expanded "><a href="async-await.html"><strong aria-hidden="true">5.</strong> Using async and await</a></li><li class="chapter-item expanded "><a href="parallelism.html"><strong aria-hidden="true">6.</strong> Parallelism</a></li><li class="chapter-item expanded "><a href="free-threading.html"><strong aria-hidden="true">7.</strong> Supporting Free-Threaded Python</a></li><li class="chapter-item expanded "><a href="debugging.html"><strong aria-hidden="true">8.</strong> Debugging</a></li><li class="chapter-item expanded "><a href="features.html"><strong aria-hidden="true">9.</strong> Features reference</a></li><li class="chapter-item expanded "><a href="performance.html"><strong aria-hidden="true">10.</strong> Performance</a></li><li class="chapter-item expanded "><a href="type-stub.html"><strong aria-hidden="true">11.</strong> Type stub generation and introspection</a></li><li class="chapter-item expanded "><a href="advanced.html"><strong aria-hidden="true">12.</strong> Advanced topics</a></li><li class="chapter-item expanded "><a href="building-and-distribution.html"><strong aria-hidden="true">13.</strong> Building and distribution</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="building-and-distribution/multiple-python-versions.html"><strong aria-hidden="true">13.1.</strong> Supporting multiple Python versions</a></li></ol></li><li class="chapter-item expanded "><a href="ecosystem.html"><strong aria-hidden="true">14.</strong> Useful crates</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="ecosystem/logging.html"><strong aria-hidden="true">14.1.</strong> Logging</a></li><li class="chapter-item expanded "><a href="ecosystem/tracing.html"><strong aria-hidden="true">14.2.</strong> Tracing</a></li><li class="chapter-item expanded "><a href="ecosystem/async-await.html"><strong aria-hidden="true">14.3.</strong> Using async and await</a></li></ol></li><li class="chapter-item expanded "><a href="faq.html"><strong aria-hidden="true">15.</strong> FAQ and troubleshooting</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded affix "><a href="migration.html">Appendix A: Migration guide</a></li><li class="chapter-item expanded affix "><a href="trait-bounds.html">Appendix B: Trait bounds</a></li><li class="chapter-item expanded affix "><a href="python-typing-hints.html">Appendix C: Python typing hints</a></li><li class="chapter-item expanded affix "><a href="changelog.html">CHANGELOG</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded affix "><a href="contributing.html">Contributing</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
