import { defineStore } from "pinia";
import { DEFAULT_USER } from "../types/user";
import { IGitProject } from "../types/gitProject";

export const useAppStore = defineStore('app', {
    state: () => (
        {
        title: "BranchWise",
        user: DEFAULT_USER,
        projects: [] as IGitProject[],
        isNavbarOpen: false,
        selectedProject: null as IGitProject | null
    }),
    getters: {
        getProjects(): IGitProject[] {
            return this.projects;
        },
        getSelectedProject(): IGitProject | null {
            return this.selectedProject;
        },
    },
    actions: {
        setTitle(title: string) {
            this.title = title;
        },
        addProject(git: IGitProject) {
            this.projects.push(git);
        },
        setProjects(projects: IGitProject[]) {
            this.projects = projects;
        },
        removeProject(git: IGitProject | null = null) {
            if (git === null && this.selectedProject !== null) {
                git = this.selectedProject;
            } else if (this.selectedProject === null) {
                return;
            }

            const index = this.projects.indexOf(git as IGitProject);
            if (index > -1) {
                this.projects.splice(index, 1);
            }

            console.log("Removed project", git);
        },
        toggleNavbar() {
            this.isNavbarOpen = !this.isNavbarOpen;
        },
        setCurrentProject(git: IGitProject | null) {
            this.selectedProject = git;
        }
    }
});
