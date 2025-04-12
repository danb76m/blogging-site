This was the start of my rust journy...

I wanted to create a blogging-site and this is pretty much the backend of it.

It's not completed, and I am archiving it because I've learnt more about rust and general programming since last touching the project (so I know better & lack of interest).


**Disclaimer>** This is just the backend. Also some of the code might be missing from the repository. 

I am using actix-web as a middlewere for authenticating, storing and retrieving files (images etc) and publishing, editing, archiving and retrieving posts.

The image upload & retrieve uses MinIO which is similiar to Amazon S3 storage (basically self hosted S3). For an added layer of security there is a checksum for the image which must match the checksum in the URL request to retrieve the image.

For authenticating, the idea was that it would only be me that would have an account as it's just a personal blogging site, so it actually uses github to authenticate so I didn't have to handle all of the logging in, resetting password etc. 

The users are stored in MongoDB. I believe posts are also stored in MongoDB. Originally however they were going to be in elasticsearch so I could easily search and retrieve posts with specific content.

