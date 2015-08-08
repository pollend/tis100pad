[![Stories in Ready](https://badge.waffle.io/pollend/tis100pad.png?label=ready&title=Ready)](https://waffle.io/pollend/tis100pad)
# tis100pad

[![Stories in Ready](https://badge.waffle.io/pollend/tis100pad.svg?label=ready&title=Ready)](http://waffle.io/pollend/tis100pad)

A web site for uploading and sharing solutions for the game TIS-100. http://www.tis100pad.com

##Installing

###Required Libraries

npm libraries

```
npm install -g grunt-cli
npm install -g bower
```

pip libraries

```
pip install flask
pip install flask-sqlalchemy
pip install Flask-KVSession
pip install Redis
```

###Configuration

```
npm install
bower install
grunt build
```
grunt will minify the javascript, and move source files from src to bin. first run the the shell.py file, this will create the tables for the server. then run the run.py to actually run the server.

##Modifying

```
grunt wtch
```

grunt will watch the src directory for any changes and rebuild the the project accordingly. Flask will restart with any changes to to the base server files but may occasionally crash with syntax errors.

```
grunt clean-up-bin
```

This command will clear the build directory file.

