create DATABASE rss;
USE rss;

CREATE TABLE user(
	uid INT PRIMARY KEY
);

CREATE TABLE channel(
	cid INT PRIMARY KEY AUTO_INCREMENT,
	uid INT,
	name varchar(50),
	FOREIGN KEY (uid) REFERENCES user(uid)
);

CREATE TABLE publisher(
	pid INT PRIMARY KEY AUTO_INCREMENT,
	url VARCHAR(500) UNIQUE,
	name VARCHAR(50)
);

CREATE TABLE subscription (
	cid INT,
	pid INT,
	PRIMARY KEY (cid, pid),
	FOREIGN KEY (cid) REFERENCES channel(cid) ON DELETE CASCADE,
	FOREIGN KEY (pid) REFERENCES publisher(pid)
);

CREATE TABLE post (
	id INT PRIMARY KEY AUTO_INCREMENT,
	url VARCHAR(500) UNIQUE,
	title VARCHAR(200),
	content TEXT,
	date_added DATETIME,
	description VARCHAR(500),
	image VARCHAR(500),
	pid INT,
	FOREIGN KEY (pid) REFERENCES publisher(pid)
);

INSERT into user (uid) VALUES (1);
INSERT into channel(uid, name) values (1, "Arjun's main feed");
