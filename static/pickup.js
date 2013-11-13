
// on ready
$(function() {

    // tiny jquery plugin for posting JSON
    // http://rohanradio.com/blog/2011/02/22/posting-json-with-jquery/
    jQuery.extend({
        postJSON: function(url, data, callback) {
            return jQuery.ajax({
                type: "POST",
                   url: url,
                   data: JSON.stringify(data),
                   success: callback,
                   dataType: "json",
                   contentType: "application/json",
                   processData: false  // don't form-encode the data
            });
        }
    });

    var t0 = new Date();
    window.App = {};

    initArtists(App);
    initAlbums(App);
    initCategories();
    initControls(App);
    initPlaylist(App);
    initRoutes(App);

    var t1 = new Date();
    // Start the backbone router / history after we retrieve the artists
    // collection.
    $.ajax("/categories/", {
        contentType: 'application/json',
        dataType: 'json',
        success: function(data) {
            var t2 = new Date();
            App.categories = {};
            console.log("Fetched all categories in %d ms", t2-t1);
            App.categories = new App.Categories(data);
            console.log("Created category collection with %d members",
                App.categories.size());
            App.categoriesView = new App.CategoriesView({
                collection:App.categories});
            App.categoriesView.render();
            console.log("Initiating the router");
            App.router = new App.Router();
            Backbone.history.start();

            /*
               console.log("Initiating the router");
               App.router = new App.Router();
               Backbone.history.start();
            // render the artist list in the background, in case it's not
            // the first thing we're looking at
            setTimeout(function() {
            //App.collectionView.renderDontShow();
            App.collectionsView.render();
            console.log("Rendered all artists");
            }, 10);
            */
        }
    });


    /*
       App.categories = new App.Categories();
       App.categories.fetch({
       success: function(data) {
       var t2 = new Date();
       App.categories = {}
       console.log("Fetched all categories in %d ms", t2-t1);
       App.categories = new App.Categories(data);
       console.log("Created category collection with %d members",
       App.categories.size());
       App.categoriesView = new App.CategoriesView({
       collection:App.categories});
       App.categoriesView.render();
       }
       });
       */

    // Set up player controls
    console.log("Initializing controls");
    App.control = new App.Control();
    App.controlView = new App.ControlView({model:App.control});
    App.control.fetch();
    // Set up polling for control status
    setInterval(function() {
        //console.log("Fetching controls");
        App.control.fetch();
    }, 10000);

    // Setup playlist polling
    App.playlist = new App.Playlist();
    App.playlistView = new App.PlaylistView({collection:App.playlist});
    App.playlist.fetch();
    setInterval(function() {
        //console.log("Fetching playlist");
        App.playlist.fetch()
    }, 10000);
});
