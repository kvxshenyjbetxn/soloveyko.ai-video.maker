Search Images
GEThttps://pixabay.com/api/
Parameters
key (required)	str	Your API key: 37888094-563bb0d2c218af7fbfff9062d
q	str	A URL encoded search term. If omitted, all images are returned. This value may not exceed 100 characters.
Example: "yellow+flower"
lang	str	Language code of the language to be searched in.
Accepted values: cs, da, de, en, es, fr, id, it, hu, nl, no, pl, pt, ro, sk, fi, sv, tr, vi, th, bg, ru, el, ja, ko, zh
Default: "en"
id	str	Retrieve individual images by ID.
image_type	str	Filter results by image type.
Accepted values: "all", "photo", "illustration", "vector"
Default: "all"
orientation	str	Whether an image is wider than it is tall, or taller than it is wide.
Accepted values: "all", "horizontal", "vertical"
Default: "all"
category	str	Filter results by category.
Accepted values: backgrounds, fashion, nature, science, education, feelings, health, people, religion, places, animals, industry, computer, food, sports, transportation, travel, buildings, business, music
min_width	int	Minimum image width.
Default: "0"
min_height	int	Minimum image height.
Default: "0"
colors	str	Filter images by color properties. A comma separated list of values may be used to select multiple properties.
Accepted values: "grayscale", "transparent", "red", "orange", "yellow", "green", "turquoise", "blue", "lilac", "pink", "white", "gray", "black", "brown"
editors_choice	bool	Select images that have received an Editor's Choice award.
Accepted values: "true", "false"
Default: "false"
safesearch	bool	A flag indicating that only images suitable for all ages should be returned.
Accepted values: "true", "false"
Default: "false"
order	str	How the results should be ordered.
Accepted values: "popular", "latest"
Default: "popular"
page	int	Returned search results are paginated. Use this parameter to select the page number.
Default: 1
per_page	int	Determine the number of results per page.
Accepted values: 3 - 200
Default: 20
callback	string	JSONP callback function name
pretty	bool	Indent JSON output. This option should not be used in production.
Accepted values: "true", "false"
Default: "false"
Example
Retrieving photos of "yellow flowers". The search term q needs to be URL encoded:

https://pixabay.com/api/?key=37888094-563bb0d2c218af7fbfff9062d&q=yellow+flowers&image_type=photo
Response for this request:

{
"total": 4692,
"totalHits": 500,
"hits": [
    {
        "id": 195893,
        "pageURL": "https://pixabay.com/en/blossom-bloom-flower-195893/",
        "type": "photo",
        "tags": "blossom, bloom, flower",
        "previewURL": "https://cdn.pixabay.com/photo/2013/10/15/09/12/flower-195893_150.jpg"
        "previewWidth": 150,
        "previewHeight": 84,
        "webformatURL": "https://pixabay.com/get/35bbf209e13e39d2_640.jpg",
        "webformatWidth": 640,
        "webformatHeight": 360,
        "largeImageURL": "https://pixabay.com/get/ed6a99fd0a76647_1280.jpg",
        "fullHDURL": "https://pixabay.com/get/ed6a9369fd0a76647_1920.jpg",
        "imageURL": "https://pixabay.com/get/ed6a9364a9fd0a76647.jpg",
        "imageWidth": 4000,
        "imageHeight": 2250,
        "imageSize": 4731420,
        "views": 7671,
        "downloads": 6439,
        "likes": 5,
        "comments": 2,
        "user_id": 48777,
        "user": "Josch13",
        "userImageURL": "https://cdn.pixabay.com/user/2013/11/05/02-10-23-764_250x250.jpg",
    },
    {
        "id": 73424,
        ...
    },
    ...
]
}
Response key	Description
total	The total number of hits.
totalHits	The number of images accessible through the API. By default, the API is limited to return a maximum of 500 images per query.
id	A unique identifier for this image.
pageURL	Source page on Pixabay, which provides a download link for the original image of the dimension imageWidth x imageHeight and the file size imageSize.
previewURL	Low resolution images with a maximum width or height of 150 px (previewWidth x previewHeight).
webformatURL	
Medium sized image with a maximum width or height of 640 px (webformatWidth x webformatHeight). URL valid for 24 hours.

Replace '_640' in any webformatURL value to access other image sizes:
Replace with '_180' or '_340' to get a 180 or 340 px tall version of the image, respectively. Replace with '_960' to get the image in a maximum dimension of 960 x 720 px.
largeImageURL	Scaled image with a maximum width/height of 1280px.
views	Total number of views.
downloads	Total number of downloads.
likes	Total number of likes.
comments	Total number of comments.
user_id, user	User ID and name of the contributor. Profile URL: https://pixabay.com/users/{ USERNAME }-{ ID }/
userImageURL	Profile picture URL (250 x 250 px).

The following response key/value pairs are only available if your account has been approved for full API access. These URLs give you access to the original images in full resolution and - if available - in vector format:

Response key	Description
fullHDURL	Full HD scaled image with a maximum width/height of 1920px.
imageURL	URL to the original image (imageWidth x imageHeight).
vectorURL	URL to a vector resource if available, else omitted.










Search Videos
GEThttps://pixabay.com/api/videos/
Parameters
key (required)	str	Your API key: 37888094-563bb0d2c218af7fbfff9062d
q	str	A URL encoded search term. If omitted, all videos are returned. This value may not exceed 100 characters.
Example: "yellow+flower"
lang	str	Language code of the language to be searched in.
Accepted values: cs, da, de, en, es, fr, id, it, hu, nl, no, pl, pt, ro, sk, fi, sv, tr, vi, th, bg, ru, el, ja, ko, zh
Default: "en"
id	str	Retrieve individual videos by ID.
video_type	str	Filter results by video type.
Accepted values: "all", "film", "animation"
Default: "all"
category	str	Filter results by category.
Accepted values: backgrounds, fashion, nature, science, education, feelings, health, people, religion, places, animals, industry, computer, food, sports, transportation, travel, buildings, business, music
min_width	int	Minimum video width.
Default: "0"
min_height	int	Minimum video height.
Default: "0"
editors_choice	bool	Select videos that have received an Editor's Choice award.
Accepted values: "true", "false"
Default: "false"
safesearch	bool	A flag indicating that only videos suitable for all ages should be returned.
Accepted values: "true", "false"
Default: "false"
order	str	How the results should be ordered.
Accepted values: "popular", "latest"
Default: "popular"
page	int	Returned search results are paginated. Use this parameter to select the page number.
Default: 1
per_page	int	Determine the number of results per page.
Accepted values: 3 - 200
Default: 20
callback	string	JSONP callback function name
pretty	bool	Indent JSON output. This option should not be used in production.
Accepted values: "true", "false"
Default: "false"
Example
Retrieving videos about "yellow flowers". The search term q needs to be URL encoded.

https://pixabay.com/api/videos/?key=37888094-563bb0d2c218af7fbfff9062d&q=yellow+flowers
Response for this request:

{
"total": 42,
"totalHits": 42,
"hits": [
    {
        "id": 125,
        "pageURL": "https://pixabay.com/videos/id-125/",
        "type": "film",
        "tags": "flowers, yellow, blossom",
        "duration": 12,
        "videos": {
            "large": {
                "url": "https://cdn.pixabay.com/video/2015/08/08/125-135736646_large.mp4",
                "width": 1920,
                "height": 1080,
                "size": 6615235,
                "thumbnail": "https://cdn.pixabay.com/video/2015/08/08/125-135736646_large.jpg"
            },
            "medium": {
                "url": "https://cdn.pixabay.com/video/2015/08/08/125-135736646_medium.mp4",
                "width": 1280,
                "height": 720,
                "size": 3562083,
                "thumbnail": "https://cdn.pixabay.com/video/2015/08/08/125-135736646_medium.jpg"
            },
            "small": {
                "url": "https://cdn.pixabay.com/video/2015/08/08/125-135736646_small.mp4",
                "width": 640,
                "height": 360,
                "size": 1030736,
                "thumbnail": "https://cdn.pixabay.com/video/2015/08/08/125-135736646_small.jpg"
            },
            "tiny": {
                "url": "https://cdn.pixabay.com/video/2015/08/08/125-135736646_tiny.mp4",
                "width": 480,
                "height": 270,
                "size": 426799,
                "thumbnail": "https://cdn.pixabay.com/video/2015/08/08/125-135736646_tiny.jpg"
            }
        },
        "views": 4462,
        "downloads": 1464,
        "likes": 18,
        "comments": 0,
        "user_id": 1281706,
        "user": "Coverr-Free-Footage",
        "userImageURL": "https://cdn.pixabay.com/user/2015/10/16/09-28-45-303_250x250.png"
    },
    {
        "id": 473,
        ...
    },
    ...
]
}
Response key	Description
total	The total number of hits.
totalHits	The number of videos accessible through the API. By default, the API is limited to return a maximum of 500 videos per query.
id	A unique identifier for this video.
pageURL	Source page on Pixabay.
videos	
A set of differently sizes video streams:

large usually has a dimension of 3840x2160. If a large video version is not available, an empty URL value and a size of zero is returned.
medium usually has a dimension of 1920x1080, older videos have a dimension of 1280x720. This size is available for all Pixabay videos.
small typically has a dimension of 1280x720, older videos have a dimension of 960x540. This size is available for all videos.
tiny typically has a dimension of 960x540, older videos have a dimension of 640x360. This size is available for all videos.
Object key	Description
url	The video URL. Append the GET parameter download=1 to the value to have the browser download it.
width	The width of the video and thumbnail.
height	The height of the video and thumbnail.
size	The approximate size of the video in bytes.
thumbnail	The URL of the poster image for this rendition.
views	Total number of views.
downloads	Total number of downloads.
likes	Total number of likes.
comments	Total number of comments.
user_id, user	User ID and name of the contributor. Profile URL: https://pixabay.com/users/{ USERNAME }-{ ID }/
userImageURL	Profile picture URL (250 x 250 px).
JavaScript Example
var API_KEY = '37888094-563bb0d2c218af7fbfff9062d';
var URL = "https://pixabay.com/api/?key="+API_KEY+"&q="+encodeURIComponent('red roses');
$.getJSON(URL, function(data){
if (parseInt(data.totalHits) > 0)
    $.each(data.hits, function(i, hit){ console.log(hit.pageURL); });
else
    console.log('No hits');
});