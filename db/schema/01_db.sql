CREATE TABLE friends (
	id INTEGER NOT NULL, 
	name VARCHAR(256), 
	link VARCHAR(1024), 
	avatar VARCHAR(1024), 
	error BOOLEAN, 
	"createAt" DATETIME, 
	PRIMARY KEY (id)
);

CREATE TABLE posts (
	id INTEGER NOT NULL, 
	title VARCHAR(256), 
	created VARCHAR(256), 
	updated VARCHAR(256), 
	link VARCHAR(1024), 
	author VARCHAR(256), 
	avatar VARCHAR(1024), 
	rule VARCHAR(256), 
	"createAt" DATETIME, 
	PRIMARY KEY (id)
);