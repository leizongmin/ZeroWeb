/*
 * Test-only offline adapter for the frozen HTML5test page.
 *
 * The archived page obtains browser branding from whichbrowser.net.  That
 * service is neither part of the HTML5 feature test nor stable enough for a
 * repository regression fixture, so preserve the page's synchronous contract
 * locally instead of making the test depend on a third-party network request.
 */
(function () {
    window.__zerowebFixtureOffline = true;
    function FixtureBrowser() {}

    FixtureBrowser.prototype.toString = function () {
        return navigator.userAgent || 'ZeroWeb fixture browser';
    };

    FixtureBrowser.prototype.isBrowser = function () { return false; };
    FixtureBrowser.prototype.isDevice = function () { return false; };
    FixtureBrowser.prototype.isEngine = function () { return false; };
    FixtureBrowser.prototype.isOs = function () { return false; };
    FixtureBrowser.prototype.isType = function () { return false; };
    FixtureBrowser.prototype.browser = {};
    FixtureBrowser.prototype.device = {};
    FixtureBrowser.prototype.engine = {};
    FixtureBrowser.prototype.features = [];
    FixtureBrowser.prototype.os = {};

    window.loadWhichBrowser = function (callback) {
        window.WhichBrowser = FixtureBrowser;
        callback();
    };

    // The archived application probes its former HTTPS endpoint before showing
    // results.  The fixture server intentionally exposes only localhost HTTP;
    // its fallback is the page's normal result-rendering path.
    window.upgradeConnection = function (_success, failure) {
        failure();
    };
})();
