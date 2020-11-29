# newbound_nebula
Make your personal Newbound Network a virtually private one with Nebula. The simple UI allows you to install, configure, and maintain Nebula on all of the Newbound peers that you have admin access to. 

For more information on Nebula:
https://slack.engineering/introducing-nebula-the-open-source-global-overlay-network-from-slack/

# Dependencies
This project requires an up-to-date working installation of the Newbound software
https://github.com/mraiser/newbound

# Installation
1. Move the data/nebula and runtime/nebula folders into your Newbound installation's data and runtime folders, respectively
2. Launch the Newbound software
3. Publish the "nebula" control in the "nebula" library using the Metabot app
4. Restart the Newbound software

*Instead of moving the data/nebula and runtime/nebula folders you can create symbolic links to them, leaving your git project folder intact for easy updating*
