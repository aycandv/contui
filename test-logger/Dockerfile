FROM alpine:latest

# Install bash for better scripting
RUN apk add --no-cache bash

# Copy the logger script
COPY logger.sh /logger.sh
RUN chmod +x /logger.sh

# Run the logger
CMD ["/logger.sh"]
