#! /usr/bin/bash
cd $newspaper_absolute_path
hash_start=`sudo -u arjun git rev-parse HEAD`
sudo -u arjun git pull origin main
hash_end=`sudo -u arjun git rev-parse HEAD`

if [ "$hash_start" != "$hash_end" ]; then
    sudo docker compose up --force-recreate --build -d
    echo "updated!"
fi
    echo "done!"

# remember to solve dubious ownership issue: git config --global --add safe.directory /home/arjun/Documents/rss-reader