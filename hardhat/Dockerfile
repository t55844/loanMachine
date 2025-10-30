# Use an official Node.js image
FROM node:20-alpine AS build

# Set the working directory inside the container
WORKDIR /app

# Copy package.json and package-lock.json first for better caching
COPY package*.json ./

# Install dependencies
RUN npm install

# Copy all your Hardhat project files
COPY . .

# Expose the blockchain port
EXPOSE 8545

# The command to start the blockchain node
# --hostname 0.0.0.0 makes it accessible to other containers
CMD ["npx", "hardhat", "node", "--hostname", "0.0.0.0"]