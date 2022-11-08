function usage() {
    cat <<USAGE
    Usage: $0 [--attach] [--pull]

    Options:
	(default): create Docker image "xv6" and attach
        --attach: attach to created Docker image (useful for GDB)
        --pull:	load latest Docker image from registry
USAGE
    exit 1
}

# Calculate Root Dir
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
cd "${SCRIPT_DIR}/.."
ROOT_DIR="$(pwd)"

# Function Type
PULL=false
ATTACH=false

for arg in "$@"; do
    case $arg in
    --pull)
        PULL=true
        shift 
        ;;
    --attach)
        ATTACH=true
        shift 
        ;;
    -h | --help)
        usage # run usage function on help
        ;;
    *)
        usage # run usage function if wrong argument provided
        ;;
    esac
done

echo $ROOT_DIR

if [[ $PULL == true ]]; then
    echo "Pulling latest image from DockerHub"
    docker pull jackwolfard/cs3210:latest
elif [[ $ATTACH == true ]]; then
    echo "Attaching to container"
    docker exec -it buzz bash 
else
    echo "Starting Buzz OS Container"
    docker run --rm -it --name="buzz" -v "${ROOT_DIR}/":/buzz -w="/buzz" buzz-os
fi

