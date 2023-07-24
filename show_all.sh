for k in lib/src/tests/snapshots/*
do
    cargo insta show $k
    echo "Press enter to continue"
    read -p ""
done
